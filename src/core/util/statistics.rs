// SPDX-License-Identifier: MIT OR Apache-2.0

use std::sync::atomic::{AtomicBool, Ordering};

/// Basic trait matching the Java `StatisticsManager` interface.
pub trait StatisticsManager: Send + Sync {
    fn start_reporting(&self);
    fn stop_reporting(&self);
    fn cleanup(&self);
}

/// Simple in-memory statistics manager used for tests.
#[derive(Debug, Default)]
pub struct DefaultStatisticsManager {
    running: AtomicBool,
}

impl DefaultStatisticsManager {
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
        }
    }
}

impl StatisticsManager for DefaultStatisticsManager {
    fn start_reporting(&self) {
        self.running.store(true, Ordering::Relaxed);
    }

    fn stop_reporting(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    fn cleanup(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
