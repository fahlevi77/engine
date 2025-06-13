pub mod timer_source;

use crate::core::stream::input::input_handler::InputHandler;
use std::sync::{Arc, Mutex};
use std::fmt::Debug;

pub trait Source: Debug + Send + Sync {
    fn start(&mut self, handler: Arc<Mutex<InputHandler>>);
    fn stop(&mut self);
    fn clone_box(&self) -> Box<dyn Source>;
}

impl Clone for Box<dyn Source> {
    fn clone(&self) -> Self { self.clone_box() }
}
