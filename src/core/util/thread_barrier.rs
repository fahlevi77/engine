use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Coordinates event flow between threads similar to Siddhi's ThreadBarrier.
#[derive(Debug)]
pub struct ThreadBarrier {
    locked: Mutex<bool>,
    cvar: Condvar,
    counter: AtomicUsize,
}

impl ThreadBarrier {
    pub fn new() -> Self {
        Self {
            locked: Mutex::new(false),
            cvar: Condvar::new(),
            counter: AtomicUsize::new(0),
        }
    }

    /// Block until the barrier is unlocked and increment active thread count.
    pub fn enter(&self) {
        let mut locked = self.locked.lock().unwrap();
        while *locked {
            locked = self.cvar.wait(locked).unwrap();
        }
        self.counter.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement active thread count.
    pub fn exit(&self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }

    /// Number of active threads inside the barrier.
    pub fn get_active_threads(&self) -> usize {
        self.counter.load(Ordering::SeqCst)
    }

    /// Lock the barrier preventing new threads from entering.
    pub fn lock(&self) {
        let mut locked = self.locked.lock().unwrap();
        *locked = true;
    }

    /// Unlock the barrier releasing waiting threads.
    pub fn unlock(&self) {
        let mut locked = self.locked.lock().unwrap();
        *locked = false;
        self.cvar.notify_all();
    }
}

impl Default for ThreadBarrier {
    fn default() -> Self {
        Self::new()
    }
}
