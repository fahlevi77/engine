// siddhi_rust/src/core/util/executor_service.rs
// Simple executor service backed by rayon thread pool.

use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;

#[derive(Debug)]
pub struct ExecutorService {
    pool: ThreadPool,
}

impl Default for ExecutorService {
    fn default() -> Self {
        let threads = num_cpus::get().max(1);
        ExecutorService::new("executor", threads)
    }
}

impl ExecutorService {
    /// Create a new executor with the given number of worker threads.
    pub fn new(name: &str, threads: usize) -> Self {
        let name_str = name.to_string();
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .thread_name(move |i| format!("{}-{}", name_str, i))
            .build()
            .expect("failed to build thread pool");
        Self { pool }
    }

    /// Submit a task for asynchronous execution.
    pub fn execute<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pool.spawn(task);
    }

    /// Block until queued tasks complete. Rayon manages its threads so this is a no-op.
    pub fn wait_all(&self) {}
}

impl Drop for ExecutorService {
    fn drop(&mut self) {}
}
