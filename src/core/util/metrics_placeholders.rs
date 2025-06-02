// siddhi_rust/src/core/util/metrics_placeholders.rs
// Placeholders for various metrics trackers used in Siddhi.
// These would eventually be replaced by actual metrics implementations or traits.

#[derive(Debug, Clone, Default)]
pub struct LatencyTrackerPlaceholder {
    _dummy: u8, // To make it non-empty if needed
}
impl LatencyTrackerPlaceholder {
    pub fn new(_name: &str, _siddhi_app_ctx: &crate::core::config::SiddhiAppContext) -> Self { Self::default() }
}


#[derive(Debug, Clone, Default)]
pub struct ThroughputTrackerPlaceholder {
    _dummy: u8,
}
impl ThroughputTrackerPlaceholder {
    pub fn new(_name: &str, _siddhi_app_ctx: &crate::core::config::SiddhiAppContext) -> Self { Self::default() }
    // pub fn event_in(&self) {}
    // pub fn events_in(&self, count: u32) {}
}


#[derive(Debug, Clone, Default)]
pub struct BufferedEventsTrackerPlaceholder {
    _dummy: u8,
}
impl BufferedEventsTrackerPlaceholder {
    pub fn new(_siddhi_app_ctx: &crate::core::config::SiddhiAppContext, _name: &str, _is_snapshotable: bool) -> Self { Self::default() }
    // pub fn increment_event_count_by(&self, count: i32) {}
    // pub fn get_size(&self) -> i64 {0}
}
