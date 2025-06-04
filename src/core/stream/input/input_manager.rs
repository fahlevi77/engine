use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::input_distributor::InputDistributor;
use super::input_entry_valve::InputEntryValve;
use super::input_handler::{InputHandler, InputProcessor};
use crate::core::config::siddhi_app_context::{SiddhiAppContext, ThreadBarrierPlaceholder};
use crate::core::stream::stream_junction::{StreamJunction, Publisher};

#[derive(Debug)]
pub struct InputManager {
    siddhi_app_context: Arc<SiddhiAppContext>,
    input_handlers: Mutex<HashMap<String, Arc<Mutex<InputHandler>>>>,
    stream_junction_map: HashMap<String, Arc<Mutex<StreamJunction>>>,
    input_distributor: Arc<Mutex<InputDistributor>>,
    input_entry_valve: Arc<Mutex<dyn InputProcessor>>,
    is_connected: Mutex<bool>,
}

impl InputManager {
    pub fn new(
        siddhi_app_context: Arc<SiddhiAppContext>,
        stream_junction_map: HashMap<String, Arc<Mutex<StreamJunction>>>,
    ) -> Self {
        let distributor: Arc<Mutex<InputDistributor>> = Arc::new(Mutex::new(InputDistributor::default()));
        let distributor_for_valve: Arc<Mutex<dyn InputProcessor>> = distributor.clone();
        let barrier_val = siddhi_app_context.thread_barrier.clone().unwrap_or_default();
        let barrier = Arc::new(barrier_val);
        let entry_valve: Arc<Mutex<dyn InputProcessor>> = Arc::new(Mutex::new(InputEntryValve::new(barrier, distributor_for_valve)));
        Self {
            siddhi_app_context,
            input_handlers: Mutex::new(HashMap::new()),
            stream_junction_map,
            input_distributor: distributor,
            input_entry_valve: entry_valve,
            is_connected: Mutex::new(false),
        }
    }

    pub fn construct_input_handler(&self, stream_id: &str) -> Result<Arc<Mutex<InputHandler>>, String> {
        let junction = self
            .stream_junction_map
            .get(stream_id)
            .ok_or_else(|| format!("StreamJunction '{}' not found", stream_id))?
            .clone();
        let publisher = junction.lock().map_err(|_| "junction mutex".to_string())?.construct_publisher();
        self.input_distributor
            .lock()
            .map_err(|_| "distributor mutex".to_string())?
            .add_input_processor(Arc::new(Mutex::new(publisher.clone())));
        let handler = Arc::new(Mutex::new(InputHandler::new(
            stream_id.to_string(),
            self.input_handlers.lock().unwrap().len(),
            self.input_entry_valve.clone(),
            Arc::clone(&self.siddhi_app_context),
        )));
        self.input_handlers
            .lock()
            .unwrap()
            .insert(stream_id.to_string(), Arc::clone(&handler));
        Ok(handler)
    }

    pub fn get_input_handler(&self, stream_id: &str) -> Option<Arc<Mutex<InputHandler>>> {
        if let Some(h) = self.input_handlers.lock().unwrap().get(stream_id) {
            return Some(h.clone());
        }
        self.construct_input_handler(stream_id).ok()
    }

    pub fn connect(&self) {
        for handler in self.input_handlers.lock().unwrap().values() {
            if let Ok(mut h) = handler.lock() {
                h.connect();
            }
        }
        *self.is_connected.lock().unwrap() = true;
    }

    pub fn disconnect(&self) {
        for handler in self.input_handlers.lock().unwrap().values() {
            if let Ok(mut h) = handler.lock() {
                h.disconnect();
            }
        }
        *self.is_connected.lock().unwrap() = false;
    }
}
