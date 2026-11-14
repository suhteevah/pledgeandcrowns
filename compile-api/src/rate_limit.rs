// SPDX-License-Identifier: AGPL-3.0-or-later
//! In-process per-client token-bucket rate limiter.
//!
//! Hand-rolled instead of pulling `tower_governor` (+ `governor`,
//! `dashmap`, `quanta`, ...): the whole thing is one mutex-guarded map and
//! ~60 lines of arithmetic, and it lets us key IPv6 clients by /64 and
//! bound memory explicitly.
//!
//! # Semantics
//!
//! Each key gets a bucket of capacity `per_minute` that refills
//! continuously at `per_minute / 60` tokens per second. A request costs
//! one token. When the bucket is empty the caller gets
//! [`Decision::Deny`] with the number of whole seconds until a token is
//! available (the HTTP `Retry-After` value).
//!
//! # Keying
//!
//! IPv4 clients are keyed by full address. IPv6 clients are keyed by
//! their /64 prefix: a single subscriber usually owns a whole /64, so
//! per-address keying would let one client rotate through 2^64 buckets.
//!
//! # Memory bound
//!
//! At most [`MAX_TRACKED_CLIENTS`] buckets are kept. When the map is full,
//! buckets that have refilled to capacity (i.e. idle clients, which carry
//! no state worth keeping) are pruned. If the map is still full after
//! pruning — a flood from more distinct clients than the bound — new
//! clients are denied (fail closed) and a warning is logged.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::Mutex;
use std::time::Instant;

/// Upper bound on distinct client buckets held in memory (~5 MB).
pub const MAX_TRACKED_CLIENTS: usize = 100_000;

/// Outcome of a rate-limit check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny { retry_after_secs: u64 },
}

#[derive(Debug, Clone, Copy)]
struct Bucket {
    tokens: f64,
    last: Instant,
}

/// Per-key token-bucket limiter. Cheap to share behind an `Arc`.
pub struct RateLimiter {
    name: &'static str,
    capacity: f64,
    refill_per_sec: f64,
    max_entries: usize,
    buckets: Mutex<HashMap<IpAddr, Bucket>>,
}

impl RateLimiter {
    /// `per_minute` must be >= 1 (enforced by [`crate::config::Config`]).
    pub fn new(name: &'static str, per_minute: u32) -> Self {
        Self::with_max_entries(name, per_minute, MAX_TRACKED_CLIENTS)
    }

    pub fn with_max_entries(name: &'static str, per_minute: u32, max_entries: usize) -> Self {
        let per_minute = f64::from(per_minute.max(1));
        Self {
            name,
            capacity: per_minute,
            refill_per_sec: per_minute / 60.0,
            max_entries: max_entries.max(1),
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Check-and-consume one token for `ip` now.
    pub fn check(&self, ip: IpAddr) -> Decision {
        self.check_at(ip, Instant::now())
    }

    /// Same as [`RateLimiter::check`] with an explicit clock, for tests.
    pub fn check_at(&self, ip: IpAddr, now: Instant) -> Decision {
        let key = bucket_key(ip);
        // A poisoned mutex only means another request panicked mid-update;
        // the map is still structurally valid, so keep serving.
        let mut map = self
            .buckets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if !map.contains_key(&key) && map.len() >= self.max_entries {
            let before = map.len();
            map.retain(|_, b| self.refilled(b, now) < self.capacity);
            tracing::debug!(
                limiter = self.name,
                pruned = before - map.len(),
                remaining = map.len(),
                "rate_limit: pruned idle buckets"
            );
            if map.len() >= self.max_entries {
                tracing::warn!(
                    limiter = self.name,
                    tracked = map.len(),
                    "rate_limit: client table full, denying new client (fail closed)"
                );
                return Decision::Deny {
                    retry_after_secs: 60,
                };
            }
        }

        let bucket = map.entry(key).or_insert(Bucket {
            tokens: self.capacity,
            last: now,
        });
        bucket.tokens = self.refilled(bucket, now);
        bucket.last = now;
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Decision::Allow
        } else {
            let deficit = 1.0 - bucket.tokens;
            let secs = (deficit / self.refill_per_sec).ceil();
            // Clamp to [1, 60]: never tell a client to retry "now", and a
            // bucket of capacity >= 1 always yields a token within 60 s.
            let retry_after_secs = if secs.is_finite() {
                (secs as u64).clamp(1, 60)
            } else {
                60
            };
            Decision::Deny { retry_after_secs }
        }
    }

    /// Number of client buckets currently tracked.
    pub fn tracked(&self) -> usize {
        self.buckets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    fn refilled(&self, b: &Bucket, now: Instant) -> f64 {
        let elapsed = now.saturating_duration_since(b.last).as_secs_f64();
        (b.tokens + elapsed * self.refill_per_sec).min(self.capacity)
    }
}

/// IPv4 (and IPv4-mapped IPv6) by full address; native IPv6 by /64.
fn bucket_key(ip: IpAddr) -> IpAddr {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn v4(last: u8) -> IpAddr {
        IpAddr::from([10, 0, 0, last])
    }

    #[test]
    fn allows_up_to_capacity_then_denies() {
        let rl = RateLimiter::new("t", 3);
        let t0 = Instant::now();
        for _ in 0..3 {
            assert_eq!(rl.check_at(v4(1), t0), Decision::Allow);
        }
        match rl.check_at(v4(1), t0) {
            Decision::Deny { retry_after_secs } => assert_eq!(retry_after_secs, 20),
            Decision::Allow => panic!("4th request in the same instant must be denied"),
        }
    }

    #[test]
    fn refills_over_time() {
        let rl = RateLimiter::new("t", 60); // 1 token / s
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
        let rl = RateLimiter::new("t", 1);
        let t0 = Instant::now();
        assert_eq!(rl.check_at(v4(1), t0), Decision::Allow);
        assert!(matches!(rl.check_at(v4(1), t0), Decision::Deny { .. }));
        assert_eq!(rl.check_at(v4(2), t0), Decision::Allow);
    }

    #[test]
    fn ipv6_keyed_by_slash_64() {
        let rl = RateLimiter::new("t", 1);
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
    fn table_full_prunes_idle_then_fails_closed() {
        let rl = RateLimiter::with_max_entries("t", 60, 2);
        let t0 = Instant::now();
        assert_eq!(rl.check_at(v4(1), t0), Decision::Allow);
        assert_eq!(rl.check_at(v4(2), t0), Decision::Allow);
        // Both buckets are partially drained -> nothing to prune -> deny.
        assert!(matches!(rl.check_at(v4(3), t0), Decision::Deny { .. }));
        // After a minute both refill to capacity -> pruned -> admitted.
        let later = t0 + Duration::from_secs(61);
        assert_eq!(rl.check_at(v4(3), later), Decision::Allow);
        assert_eq!(rl.tracked(), 1);
    }
}
