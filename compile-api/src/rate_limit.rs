// SPDX-License-Identifier: AGPL-3.0-or-later
//! In-process per-client token-bucket rate limiter.
//!
//! Hand-rolled instead of pulling `tower_governor` (+ `governor`,
//! `dashmap`, `quanta`, ...): one mutex-guarded map and some arithmetic,
//! and it lets us key IPv6 by prefix and bound memory explicitly.
//!
//! # Semantics
//!
//! Each key gets a bucket of capacity `per_minute` that refills
//! continuously at `per_minute / 60` tokens per second. A request costs
//! one token. When a bucket is empty the caller gets [`Decision::Deny`]
//! with the whole seconds until a token is available (`Retry-After`).
//!
//! # Keying
//!
//! - IPv4 (and IPv4-mapped IPv6): full address.
//! - Native IPv6: the /64 (one subscriber usually owns a whole /64, so
//!   per-address keying would hand one client 2^64 buckets), **and** an
//!   aggregate bucket per /48 with [`V6_AGGREGATE_MULTIPLIER`] times the
//!   capacity, so a client that owns a /48 cannot rotate through 65 536
//!   fresh /64 buckets. A request must pass both.
//!
//! # Memory bound and pressure
//!
//! At most `max_entries` buckets per table. When a new key arrives at a
//! full table, idle buckets (refilled to capacity, so carrying no state)
//! are pruned — at most once per [`PRUNE_INTERVAL`] so a flood cannot make
//! every request pay an O(n) scan. If the table is still full the
//! limiter's [`OnFull`] policy decides:
//!
//! - [`OnFull::FailOpen`] (global limiter): the request is allowed without
//!   tracking a bucket and counted as a fail-open. Already-tracked clients
//!   stay limited. A distributed flood large enough to fill the table
//!   makes a per-IP limit meaningless anyway; refusing every *new* client
//!   (the old fail-closed behaviour) turned that flood into an outage.
//! - [`OnFull::EvictLru`] (`/compile-real`): the least-recently-seen
//!   1/8 of the table is evicted in one batch (amortised O(1) per new key)
//!   and the new client gets a bucket. That route is also bounded by the
//!   build semaphore and queue, so a reset bucket cannot overload it.
//!
//! The old behaviour — deny every new client once full, on every route
//! including `/health` — is gone. `/health` is also mounted outside the
//! limiter entirely (see `build_router`).

use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Default upper bound on distinct client buckets per table (~5 MB).
pub const MAX_TRACKED_CLIENTS: usize = 100_000;

/// Capacity of a /48 aggregate bucket relative to a single client's.
pub const V6_AGGREGATE_MULTIPLIER: u32 = 16;

/// Minimum spacing between idle-bucket prune scans of a full table.
pub const PRUNE_INTERVAL: Duration = Duration::from_secs(1);

/// What a full table does with a new key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnFull {
    FailOpen,
    EvictLru,
}

/// Outcome of a rate-limit check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    /// Allowed untracked because the table is under pressure.
    FailOpen,
    Deny {
        retry_after_secs: u64,
    },
}

/// Counters exported to `/metrics`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LimiterStats {
    pub tracked: usize,
    pub evictions: u64,
    pub fail_opens: u64,
}

#[derive(Debug, Clone, Copy)]
struct Bucket {
    tokens: f64,
    last: Instant,
}

struct Table {
    map: HashMap<IpAddr, Bucket>,
    capacity: f64,
    refill_per_sec: f64,
    max_entries: usize,
    last_prune: Option<Instant>,
}

enum Slot {
    Ready,
    Full,
}

impl Table {
    fn new(per_minute: f64, max_entries: usize) -> Self {
        Self {
            map: HashMap::new(),
            capacity: per_minute,
            refill_per_sec: per_minute / 60.0,
            max_entries: max_entries.max(1),
            last_prune: None,
        }
    }

    fn refilled(&self, b: &Bucket, now: Instant) -> f64 {
        let elapsed = now.saturating_duration_since(b.last).as_secs_f64();
        (b.tokens + elapsed * self.refill_per_sec).min(self.capacity)
    }

    /// Make room for `key` if needed, pruning idle buckets (rate-limited).
    fn ensure_slot(&mut self, key: &IpAddr, now: Instant, name: &str) -> Slot {
        if self.map.contains_key(key) || self.map.len() < self.max_entries {
            return Slot::Ready;
        }
        let may_prune = self
            .last_prune
            .is_none_or(|t| now.saturating_duration_since(t) >= PRUNE_INTERVAL);
        if may_prune {
            self.last_prune = Some(now);
            let before = self.map.len();
            let (cap, rate) = (self.capacity, self.refill_per_sec);
            self.map.retain(|_, b| {
                let elapsed = now.saturating_duration_since(b.last).as_secs_f64();
                (b.tokens + elapsed * rate).min(cap) < cap
            });
            tracing::debug!(
                limiter = name,
                pruned = before - self.map.len(),
                remaining = self.map.len(),
                "rate_limit: pruned idle buckets"
            );
        }
        if self.map.len() < self.max_entries {
            Slot::Ready
        } else {
            Slot::Full
        }
    }

    /// Evict the least-recently-seen ~1/8 of the table. Returns how many.
    fn evict_lru_batch(&mut self) -> usize {
        let n = (self.max_entries / 8).max(1).min(self.map.len());
        if n == 0 {
            return 0;
        }
        let mut by_age: Vec<(Instant, IpAddr)> =
            self.map.iter().map(|(k, b)| (b.last, *k)).collect();
        by_age.select_nth_unstable_by_key(n - 1, |e| e.0);
        for (_, k) in by_age.into_iter().take(n) {
            self.map.remove(&k);
        }
        n
    }

    fn tokens_now(&self, key: &IpAddr, now: Instant) -> f64 {
        self.map
            .get(key)
            .map_or(self.capacity, |b| self.refilled(b, now))
    }

    fn retry_after(&self, tokens: f64) -> u64 {
        let secs = ((1.0 - tokens) / self.refill_per_sec).ceil();
        // Clamp to [1, 60]: never tell a client to retry "now", and a
        // bucket of capacity >= 1 always yields a token within 60 s (the
        // /48 aggregate refills at least as fast).
        if secs.is_finite() {
            (secs as u64).clamp(1, 60)
        } else {
            60
        }
    }

    fn store(&mut self, key: IpAddr, tokens: f64, now: Instant) {
        self.map.insert(key, Bucket { tokens, last: now });
    }

    /// Update `key` only if it is already tracked.
    fn store_existing(&mut self, key: IpAddr, tokens: f64, now: Instant) {
        if let Some(b) = self.map.get_mut(&key) {
            *b = Bucket { tokens, last: now };
        }
    }
}

struct Tables {
    clients: Table,
    v6_aggregates: Table,
}

/// Per-key limiter. Cheap to share behind an `Arc`.
pub struct RateLimiter {
    name: &'static str,
    on_full: OnFull,
    tables: Mutex<Tables>,
    evictions: AtomicU64,
    fail_opens: AtomicU64,
}

impl RateLimiter {
    /// `per_minute` must be >= 1 (enforced by [`crate::config::Config`]).
    pub fn new(name: &'static str, per_minute: u32, on_full: OnFull) -> Self {
        Self::with_max_entries(name, per_minute, MAX_TRACKED_CLIENTS, on_full)
    }

    pub fn with_max_entries(
        name: &'static str,
        per_minute: u32,
        max_entries: usize,
        on_full: OnFull,
    ) -> Self {
        let per_minute = per_minute.max(1);
        Self {
            name,
            on_full,
            tables: Mutex::new(Tables {
                clients: Table::new(f64::from(per_minute), max_entries),
                v6_aggregates: Table::new(
                    f64::from(per_minute.saturating_mul(V6_AGGREGATE_MULTIPLIER)),
                    max_entries,
                ),
            }),
            evictions: AtomicU64::new(0),
            fail_opens: AtomicU64::new(0),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Check-and-consume one token for `ip` now.
    pub fn check(&self, ip: IpAddr) -> Decision {
        self.check_at(ip, Instant::now())
    }

    /// Same as [`RateLimiter::check`] with an explicit clock, for tests.
    pub fn check_at(&self, ip: IpAddr, now: Instant) -> Decision {
        let key = client_key(ip);
        let agg = v6_aggregate_key(ip);
        // A poisoned mutex only means another request panicked mid-update;
        // the maps are still structurally valid, so keep serving.
        let mut t = self
            .tables
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Tables {
            clients,
            v6_aggregates,
        } = &mut *t;

        // Decide first, from the stored buckets (an untracked key counts
        // as a full bucket), so a denied request never creates, evicts or
        // prunes anything: a flood of fresh /64s that the /48 aggregate
        // refuses must not fill the client table (re-review P2).
        let client_tokens = clients.tokens_now(&key, now);
        let agg_tokens = agg.as_ref().map(|a| v6_aggregates.tokens_now(a, now));

        let deny_client = client_tokens < 1.0;
        let deny_agg = agg_tokens.is_some_and(|t| t < 1.0);
        if deny_client || deny_agg {
            // Persist the refill of buckets that already exist; consume
            // nothing and store nothing new.
            clients.store_existing(key, client_tokens, now);
            if let (Some(a), Some(t)) = (agg, agg_tokens) {
                v6_aggregates.store_existing(a, t, now);
            }
            let retry_after_secs = if deny_client {
                clients.retry_after(client_tokens)
            } else {
                v6_aggregates.retry_after(agg_tokens.unwrap_or(0.0))
            };
            if deny_agg && !deny_client {
                tracing::debug!(limiter = self.name, %ip, "rate_limit: /48 aggregate exhausted");
            }
            return Decision::Deny { retry_after_secs };
        }

        if !self.make_room(clients, &key, now) {
            return Decision::FailOpen;
        }
        if let Some(agg) = &agg
            && !self.make_room(v6_aggregates, agg, now)
        {
            return Decision::FailOpen;
        }
        clients.store(key, client_tokens - 1.0, now);
        if let (Some(a), Some(t)) = (agg, agg_tokens) {
            v6_aggregates.store(a, t - 1.0, now);
        }
        Decision::Allow
    }

    /// `true` if `key` may be tracked; `false` means "fail open".
    fn make_room(&self, table: &mut Table, key: &IpAddr, now: Instant) -> bool {
        match table.ensure_slot(key, now, self.name) {
            Slot::Ready => true,
            Slot::Full => match self.on_full {
                OnFull::FailOpen => {
                    self.fail_opens.fetch_add(1, Ordering::Relaxed);
                    tracing::warn!(
                        limiter = self.name,
                        tracked = table.map.len(),
                        "rate_limit: client table full, allowing untracked request (fail open)"
                    );
                    false
                }
                OnFull::EvictLru => {
                    let n = table.evict_lru_batch();
                    self.evictions.fetch_add(n as u64, Ordering::Relaxed);
                    tracing::warn!(
                        limiter = self.name,
                        evicted = n,
                        "rate_limit: client table full, evicted least-recently-seen buckets"
                    );
                    true
                }
            },
        }
    }

    /// Number of client buckets currently tracked (excluding /48 aggregates).
    pub fn tracked(&self) -> usize {
        self.tables
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clients
            .map
            .len()
    }

    pub fn stats(&self) -> LimiterStats {
        LimiterStats {
            tracked: self.tracked(),
            evictions: self.evictions.load(Ordering::Relaxed),
            fail_opens: self.fail_opens.load(Ordering::Relaxed),
        }
    }
}

/// IPv4 (and IPv4-mapped IPv6) by full address; native IPv6 by /64.
fn client_key(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V4(_) => ip,
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return IpAddr::V4(v4);
            }
            let s = v6.segments();
            IpAddr::V6(Ipv6Addr::new(s[0], s[1], s[2], s[3], 0, 0, 0, 0))
        }
    }
}

/// Native IPv6 only: the /48 aggregate key.
fn v6_aggregate_key(ip: IpAddr) -> Option<IpAddr> {
    match ip {
        IpAddr::V6(v6) if v6.to_ipv4_mapped().is_none() => {
            let s = v6.segments();
            Some(IpAddr::V6(Ipv6Addr::new(s[0], s[1], s[2], 0, 0, 0, 0, 0)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v4(last: u8) -> IpAddr {
        IpAddr::from([10, 0, 0, last])
    }

    #[test]
    fn allows_up_to_capacity_then_denies() {
        let rl = RateLimiter::new("t", 3, OnFull::EvictLru);
        let t0 = Instant::now();
        for _ in 0..3 {
            assert_eq!(rl.check_at(v4(1), t0), Decision::Allow);
        }
        match rl.check_at(v4(1), t0) {
            Decision::Deny { retry_after_secs } => assert_eq!(retry_after_secs, 20),
            other => panic!("4th request in the same instant must be denied, got {other:?}"),
        }
    }

    #[test]
    fn refills_over_time() {
        let rl = RateLimiter::new("t", 60, OnFull::EvictLru); // 1 token / s
        let t0 = Instant::now();
        for _ in 0..60 {
            assert_eq!(rl.check_at(v4(1), t0), Decision::Allow);
        }
        assert!(matches!(rl.check_at(v4(1), t0), Decision::Deny { .. }));
        assert_eq!(
            rl.check_at(v4(1), t0 + Duration::from_millis(1_001)),
            Decision::Allow
        );
    }

    #[test]
    fn clients_are_independent() {
        let rl = RateLimiter::new("t", 1, OnFull::EvictLru);
        let t0 = Instant::now();
        assert_eq!(rl.check_at(v4(1), t0), Decision::Allow);
        assert!(matches!(rl.check_at(v4(1), t0), Decision::Deny { .. }));
        assert_eq!(rl.check_at(v4(2), t0), Decision::Allow);
    }

    #[test]
    fn ipv6_keyed_by_slash_64() {
        let rl = RateLimiter::new("t", 1, OnFull::EvictLru);
        let t0 = Instant::now();
        let a: IpAddr = "2001:db8:1:2::1".parse().unwrap();
        let b: IpAddr = "2001:db8:1:2:ffff::9".parse().unwrap();
        let c: IpAddr = "2001:db8:1:3::1".parse().unwrap();
        assert_eq!(rl.check_at(a, t0), Decision::Allow);
        assert!(
            matches!(rl.check_at(b, t0), Decision::Deny { .. }),
            "same /64 must share a bucket"
        );
        assert_eq!(rl.check_at(c, t0), Decision::Allow);
    }

    #[test]
    fn ipv6_slash_48_aggregate_caps_prefix_rotation() {
        let rl = RateLimiter::new("t", 1, OnFull::EvictLru);
        let t0 = Instant::now();
        // 16 distinct /64s inside 2001:db8:1::/48 each get their one token...
        for subnet in 0..V6_AGGREGATE_MULTIPLIER as u16 {
            let ip = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 1, subnet, 0, 0, 0, 1));
            assert_eq!(rl.check_at(ip, t0), Decision::Allow, "subnet {subnet}");
        }
        // ...the 17th fresh /64 in the same /48 is refused by the aggregate,
        let ip17 = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 1, 0x99, 0, 0, 0, 1));
        assert!(matches!(rl.check_at(ip17, t0), Decision::Deny { .. }));
        // ...while a different /48 is unaffected.
        let other: IpAddr = "2001:db8:2::1".parse().unwrap();
        assert_eq!(rl.check_at(other, t0), Decision::Allow);
        // IPv4-mapped addresses are keyed as IPv4, with no aggregate.
        let mapped: IpAddr = "::ffff:10.0.0.7".parse().unwrap();
        assert_eq!(rl.check_at(mapped, t0), Decision::Allow);
        assert!(matches!(rl.check_at(v4(7), t0), Decision::Deny { .. }));
    }

    /// Re-review P2: a /64 refused by its /48 aggregate must not get a
    /// client entry (otherwise rotating /64s fills the table for free).
    #[test]
    fn aggregate_denials_store_no_client_entry() {
        let rl = RateLimiter::new("t", 1, OnFull::EvictLru);
        let t0 = Instant::now();
        for subnet in 0..V6_AGGREGATE_MULTIPLIER as u16 {
            let ip = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 7, subnet, 0, 0, 0, 1));
            assert_eq!(rl.check_at(ip, t0), Decision::Allow);
        }
        let tracked = rl.tracked();
        assert_eq!(tracked, V6_AGGREGATE_MULTIPLIER as usize);
        for subnet in 100..1_100u16 {
            let ip = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 7, subnet, 0, 0, 0, 1));
            assert!(matches!(rl.check_at(ip, t0), Decision::Deny { .. }));
        }
        assert_eq!(rl.tracked(), tracked, "denied /64s must not be stored");
        assert_eq!(rl.stats().evictions, 0, "a denial must not evict");
    }

    #[test]
    fn full_table_evicts_lru_instead_of_refusing() {
        let rl = RateLimiter::with_max_entries("t", 60, 2, OnFull::EvictLru);
        let t0 = Instant::now();
        assert_eq!(rl.check_at(v4(1), t0), Decision::Allow);
        // v4(2) is seen later and fully drained.
        let t1 = t0 + Duration::from_millis(10);
        for _ in 0..60 {
            assert_eq!(rl.check_at(v4(2), t1), Decision::Allow);
        }
        // Both partially/fully drained -> nothing idle -> evict the oldest
        // (v4(1)) and admit the newcomer instead of refusing it.
        let t2 = t0 + Duration::from_millis(20);
        assert_eq!(rl.check_at(v4(3), t2), Decision::Allow);
        assert_eq!(rl.tracked(), 2);
        assert_eq!(rl.stats().evictions, 1);
        // The recently-seen abuser kept its (empty) bucket.
        assert!(matches!(rl.check_at(v4(2), t2), Decision::Deny { .. }));
    }

    #[test]
    fn full_table_fails_open_for_global_and_counts_it() {
        let rl = RateLimiter::with_max_entries("global", 1, 2, OnFull::FailOpen);
        let t0 = Instant::now();
        assert_eq!(rl.check_at(v4(1), t0), Decision::Allow);
        assert_eq!(rl.check_at(v4(2), t0), Decision::Allow);
        // Table full of drained buckets: a new client is let through
        // untracked, and counted.
        assert_eq!(rl.check_at(v4(3), t0), Decision::FailOpen);
        assert_eq!(rl.check_at(v4(4), t0), Decision::FailOpen);
        assert_eq!(rl.stats().fail_opens, 2);
        assert_eq!(rl.tracked(), 2);
        // Tracked clients are still limited.
        assert!(matches!(rl.check_at(v4(1), t0), Decision::Deny { .. }));
        // Once buckets go idle (and the prune interval passed) newcomers
        // are tracked again.
        let later = t0 + Duration::from_secs(61);
        assert_eq!(rl.check_at(v4(3), later), Decision::Allow);
        assert_eq!(rl.tracked(), 1);
    }

    #[test]
    fn prune_scans_are_rate_limited() {
        let rl = RateLimiter::with_max_entries("global", 60, 1, OnFull::FailOpen);
        let t0 = Instant::now();
        assert_eq!(rl.check_at(v4(1), t0), Decision::Allow);
        // v4(1) still has 59 tokens: not idle, so the prune frees nothing.
        assert_eq!(rl.check_at(v4(2), t0), Decision::FailOpen);
        // v4(1) becomes idle 1 s later, but within PRUNE_INTERVAL of the
        // last scan nothing is re-scanned...
        let t_half = t0 + Duration::from_millis(500);
        assert_eq!(rl.check_at(v4(3), t_half), Decision::FailOpen);
        // ...after the interval the idle bucket is pruned.
        let t_late = t0 + Duration::from_secs(2);
        assert_eq!(rl.check_at(v4(3), t_late), Decision::Allow);
    }
}
