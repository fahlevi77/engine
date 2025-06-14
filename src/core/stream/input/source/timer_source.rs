use super::Source;
use crate::core::event::event::Event;
use crate::core::event::value::AttributeValue;
use crate::core::stream::input::input_handler::InputHandler;
use std::fmt::Debug;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TimerSource {
    interval_ms: u64,
    running: Arc<AtomicBool>,
}

impl TimerSource {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            interval_ms,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Source for TimerSource {
    fn start(&mut self, handler: Arc<Mutex<InputHandler>>) {
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);
        let interval = self.interval_ms;
        thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                let event =
                    Event::new_with_data(0, vec![AttributeValue::String("tick".to_string())]);
                if let Ok(mut h) = handler.lock() {
                    let _ = h.send_single_event(event);
                }
                thread::sleep(Duration::from_millis(interval));
            }
        });
    }

    fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }

    fn clone_box(&self) -> Box<dyn Source> {
        Box::new(self.clone())
    }
}
