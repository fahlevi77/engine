// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::core::util::executor_service::ExecutorService;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub struct ScheduledExecutorService {
    pub executor: Arc<ExecutorService>,
}

impl ScheduledExecutorService {
    pub fn new(executor: Arc<ExecutorService>) -> Self {
        Self { executor }
    }

    pub fn schedule<F>(&self, delay_ms: u64, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let _exec = Arc::clone(&self.executor);
        self.executor.execute(move || {
            std::thread::sleep(Duration::from_millis(delay_ms));
            task();
        });
    }
}
