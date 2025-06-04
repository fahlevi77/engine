// siddhi_rust/src/core/util/executor_service.rs
// Very small thread pool used as a stand in for Java's ExecutorService.

use crossbeam_channel::{unbounded, Sender};
use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};

enum Message {
    Run(Box<dyn FnOnce() + Send + 'static>),
    Shutdown,
}

#[derive(Debug)]
pub struct ExecutorService {
    name: String,
    sender: Sender<Message>,
    workers: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl Default for ExecutorService {
    fn default() -> Self {
        ExecutorService::new("executor", 1)
    }
}

impl ExecutorService {
    /// Create a new executor with the given number of worker threads.
    pub fn new(name: &str, threads: usize) -> Self {
        let (tx, rx) = unbounded::<Message>();
        let workers = Arc::new(Mutex::new(Vec::new()));
        let shared_rx = Arc::new(Mutex::new(rx));
        for i in 0..threads {
            let rx = Arc::clone(&shared_rx);
            let handle = thread::Builder::new()
                .name(format!("{}-{}", name, i))
                .spawn(move || {
                    loop {
                        let msg = {
                            let lock = rx.lock().expect("rx mutex");
                            lock.recv()
                        };
                        match msg {
                            Ok(Message::Run(job)) => { job(); }
                            Ok(Message::Shutdown) | Err(_) => break,
                        }
                    }
                })
                .expect("thread spawn");
            workers.lock().unwrap().push(handle);
        }
        Self { name: name.to_string(), sender: tx, workers }
    }

    /// Submit a task for asynchronous execution.
    pub fn execute<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let _ = self.sender.send(Message::Run(Box::new(task)));
    }

    /// Signal all worker threads to shut down and wait for them to finish.
    pub fn shutdown(&self) {
        for _ in 0..self.workers.lock().unwrap().len() {
            let _ = self.sender.send(Message::Shutdown);
        }
        let mut workers = self.workers.lock().unwrap();
        while let Some(h) = workers.pop() {
            let _ = h.join();
        }
    }
}

impl Drop for ExecutorService {
    fn drop(&mut self) {
        self.shutdown();
    }
}

