use std::time::Duration;
use std::sync::Arc;
use crate::core::util::executor_service::ExecutorService;

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
        let exec = Arc::clone(&self.executor);
        self.executor.execute(move || {
            std::thread::sleep(Duration::from_millis(delay_ms));
            task();
        });
    }
}
