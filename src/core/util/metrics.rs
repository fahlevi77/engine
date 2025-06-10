use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::Duration;
use crate::core::config::SiddhiAppContext;

#[derive(Debug, Default)]
pub struct LatencyTracker {
    total_ns: AtomicU64,
    count: AtomicU64,
}

impl Clone for LatencyTracker {
    fn clone(&self) -> Self {
        Self {
            total_ns: AtomicU64::new(self.total_ns.load(Ordering::Relaxed)),
            count: AtomicU64::new(self.count.load(Ordering::Relaxed)),
        }
    }
}

impl LatencyTracker {
    pub fn new(_name: &str, _ctx: &SiddhiAppContext) -> Self { Self::default() }
    pub fn add_latency(&self, dur: Duration) {
        self.total_ns.fetch_add(dur.as_nanos() as u64, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }
    pub fn average_latency_ns(&self) -> Option<u64> {
        let c = self.count.load(Ordering::Relaxed);
        if c == 0 { None } else { Some(self.total_ns.load(Ordering::Relaxed)/c) }
    }
}

#[derive(Debug, Default)]
pub struct ThroughputTracker {
    count: AtomicU64,
}

impl Clone for ThroughputTracker {
    fn clone(&self) -> Self {
        Self { count: AtomicU64::new(self.count.load(Ordering::Relaxed)) }
    }
}

impl ThroughputTracker {
    pub fn new(_name: &str, _ctx: &SiddhiAppContext) -> Self { Self::default() }
    pub fn event_in(&self) { self.count.fetch_add(1, Ordering::Relaxed); }
    pub fn events_in(&self, n: u32) { self.count.fetch_add(n as u64, Ordering::Relaxed); }
    pub fn total_events(&self) -> u64 { self.count.load(Ordering::Relaxed) }
}

#[derive(Debug, Default)]
pub struct BufferedEventsTracker {
    count: AtomicI64,
}

impl Clone for BufferedEventsTracker {
    fn clone(&self) -> Self {
        Self { count: AtomicI64::new(self.count.load(Ordering::Relaxed)) }
    }
}

impl BufferedEventsTracker {
    pub fn new(_ctx: &SiddhiAppContext, _name: &str, _is_snapshotable: bool) -> Self { Self::default() }
    pub fn increment_event_count_by(&self, n: i32) { self.count.fetch_add(n as i64, Ordering::Relaxed); }
    pub fn get_size(&self) -> i64 { self.count.load(Ordering::Relaxed) }
}
