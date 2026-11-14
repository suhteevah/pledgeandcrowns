// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hand-rolled Prometheus text exposition (format 0.0.4) for `/metrics`.
//!
//! No `prometheus` crate: the metric set is small and fixed, and the text
//! format is a few `writeln!`s. All label values are drawn from closed
//! sets (route templates, HTTP status codes, a fixed reason enum), so the
//! series count is bounded and no player input can reach a label.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::Duration;

/// Prometheus text-format content type.
pub const CONTENT_TYPE: &str = "text/plain; version=0.0.4; charset=utf-8";

/// Why a request was refused before reaching a handler's real work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    Auth,
    Rate,
    Queue,
    Body,
}

impl RejectReason {
    const ALL: [RejectReason; 4] = [Self::Auth, Self::Rate, Self::Queue, Self::Body];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auth => "auth",
            Self::Rate => "rate",
            Self::Queue => "queue",
            Self::Body => "body",
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// Seconds. Builds are 1-3 s warm with a 30 s watchdog.
const BUILD_BUCKETS: &[f64] = &[0.25, 0.5, 1.0, 2.0, 3.0, 5.0, 10.0, 20.0, 30.0, 60.0];
/// Seconds. Runs are sub-second with a 5 s watchdog.
const RUN_BUCKETS: &[f64] = &[0.001, 0.005, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0];

struct HistogramData {
    /// Non-cumulative per-bucket counts; index `bounds.len()` is +Inf.
    counts: Vec<u64>,
    sum: f64,
    count: u64,
}

struct Histogram {
    bounds: &'static [f64],
    data: Mutex<HistogramData>,
}

impl Histogram {
    fn new(bounds: &'static [f64]) -> Self {
        Self {
            bounds,
            data: Mutex::new(HistogramData {
                counts: vec![0; bounds.len() + 1],
                sum: 0.0,
                count: 0,
            }),
        }
    }

    fn observe(&self, d: Duration) {
        let v = d.as_secs_f64();
        let idx = self
            .bounds
            .iter()
            .position(|b| v <= *b)
            .unwrap_or(self.bounds.len());
        let mut data = self.data.lock().unwrap_or_else(|p| p.into_inner());
        data.counts[idx] += 1;
        data.sum += v;
        data.count += 1;
    }

    fn render(&self, out: &mut String, name: &str, help: &str) {
        let data = self.data.lock().unwrap_or_else(|p| p.into_inner());
        let _ = writeln!(out, "# HELP {name} {help}");
        let _ = writeln!(out, "# TYPE {name} histogram");
        let mut cumulative = 0u64;
        for (i, bound) in self.bounds.iter().enumerate() {
            cumulative += data.counts[i];
            let _ = writeln!(out, "{name}_bucket{{le=\"{bound}\"}} {cumulative}");
        }
        cumulative += data.counts[self.bounds.len()];
        let _ = writeln!(out, "{name}_bucket{{le=\"+Inf\"}} {cumulative}");
        let _ = writeln!(out, "{name}_sum {}", data.sum);
        let _ = writeln!(out, "{name}_count {}", data.count);
    }
}

/// Process-wide metric registry. One instance lives in `AppState`.
pub struct Metrics {
    requests: Mutex<BTreeMap<(String, u16), u64>>,
    build_duration: Histogram,
    run_duration: Histogram,
    builds_in_flight: AtomicI64,
    builds_queued: AtomicI64,
    rejections: [AtomicU64; 4],
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            requests: Mutex::new(BTreeMap::new()),
            build_duration: Histogram::new(BUILD_BUCKETS),
            run_duration: Histogram::new(RUN_BUCKETS),
            builds_in_flight: AtomicI64::new(0),
            builds_queued: AtomicI64::new(0),
            rejections: Default::default(),
        }
    }

    /// `route` must be a route template (or `"unmatched"`), never a raw
    /// request path, to keep label cardinality bounded.
    pub fn record_request(&self, route: &str, status: u16) {
        let mut map = self.requests.lock().unwrap_or_else(|p| p.into_inner());
        *map.entry((route.to_string(), status)).or_insert(0) += 1;
    }

    pub fn record_rejection(&self, reason: RejectReason) {
        self.rejections[reason.index()].fetch_add(1, Ordering::Relaxed);
    }

    pub fn observe_build(&self, d: Duration) {
        self.build_duration.observe(d);
    }

    pub fn observe_run(&self, d: Duration) {
        self.run_duration.observe(d);
    }

    pub fn builds_in_flight_add(&self, delta: i64) {
        self.builds_in_flight.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn builds_queued_add(&self, delta: i64) {
        self.builds_queued.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn rejections(&self, reason: RejectReason) -> u64 {
        self.rejections[reason.index()].load(Ordering::Relaxed)
    }

    /// Render the full exposition text.
    pub fn render(&self) -> String {
        let mut out = String::with_capacity(4096);

        let _ = writeln!(
            out,
            "# HELP pnc_http_requests_total HTTP requests by route template and status code."
        );
        let _ = writeln!(out, "# TYPE pnc_http_requests_total counter");
        {
            let map = self.requests.lock().unwrap_or_else(|p| p.into_inner());
            for ((route, status), n) in map.iter() {
                let _ = writeln!(
                    out,
                    "pnc_http_requests_total{{route=\"{}\",status=\"{status}\"}} {n}",
                    escape_label(route)
                );
            }
        }

        let _ = writeln!(
            out,
            "# HELP pnc_rejections_total Requests refused before real work, by reason."
        );
        let _ = writeln!(out, "# TYPE pnc_rejections_total counter");
        for r in RejectReason::ALL {
            let _ = writeln!(
                out,
                "pnc_rejections_total{{reason=\"{}\"}} {}",
                r.as_str(),
                self.rejections(r)
            );
        }

        let _ = writeln!(
            out,
            "# HELP pnc_builds_in_flight Real builds currently executing."
        );
        let _ = writeln!(out, "# TYPE pnc_builds_in_flight gauge");
        let _ = writeln!(
            out,
            "pnc_builds_in_flight {}",
            self.builds_in_flight.load(Ordering::Relaxed)
        );

        let _ = writeln!(
            out,
            "# HELP pnc_builds_queued Real builds waiting for a build slot."
        );
        let _ = writeln!(out, "# TYPE pnc_builds_queued gauge");
        let _ = writeln!(
            out,
            "pnc_builds_queued {}",
            self.builds_queued.load(Ordering::Relaxed)
        );

        self.build_duration.render(
            &mut out,
            "pnc_build_duration_seconds",
            "Wall time of cargo build for wasm32-wasip1.",
        );
        self.run_duration.render(
            &mut out,
            "pnc_run_duration_seconds",
            "Wall time of the sandboxed wasm run.",
        );
        out
    }
}

fn escape_label(v: &str) -> String {
    v.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_buckets_are_cumulative() {
        let m = Metrics::new();
        m.observe_build(Duration::from_millis(400));
        m.observe_build(Duration::from_secs(2));
        m.observe_build(Duration::from_secs(120));
        let text = m.render();
        assert!(text.contains("pnc_build_duration_seconds_bucket{le=\"0.25\"} 0"));
        assert!(text.contains("pnc_build_duration_seconds_bucket{le=\"0.5\"} 1"));
        assert!(text.contains("pnc_build_duration_seconds_bucket{le=\"2\"} 2"));
        assert!(text.contains("pnc_build_duration_seconds_bucket{le=\"60\"} 2"));
        assert!(text.contains("pnc_build_duration_seconds_bucket{le=\"+Inf\"} 3"));
        assert!(text.contains("pnc_build_duration_seconds_count 3"));
    }

    #[test]
    fn counters_render_with_labels() {
        let m = Metrics::new();
        m.record_request("/compile", 200);
        m.record_request("/compile", 200);
        m.record_rejection(RejectReason::Rate);
        let text = m.render();
        assert!(text.contains("pnc_http_requests_total{route=\"/compile\",status=\"200\"} 2"));
        assert!(text.contains("pnc_rejections_total{reason=\"rate\"} 1"));
        assert!(text.contains("pnc_rejections_total{reason=\"auth\"} 0"));
    }
}
