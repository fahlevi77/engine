use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::stream::stream_event::StreamEvent;
use crate::core::event::value::AttributeValue;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug)]
pub struct BaseIncrementalValueStore {
    pub expression_executors: Vec<Box<dyn ExpressionExecutor>>,
    grouped_values: Mutex<HashMap<String, Vec<AttributeValue>>>,
    timestamp: Mutex<i64>,
}

impl BaseIncrementalValueStore {
    pub fn new(expression_executors: Vec<Box<dyn ExpressionExecutor>>) -> Self {
        Self {
            expression_executors,
            grouped_values: Mutex::new(HashMap::new()),
            timestamp: Mutex::new(-1),
        }
    }

    pub fn process(&self, group: &str, event: &StreamEvent) {
        let mut map = self.grouped_values.lock().unwrap();
        let entry = map
            .entry(group.to_string())
            .or_insert_with(|| vec![AttributeValue::default(); self.expression_executors.len()]);
        for (i, exec) in self.expression_executors.iter().enumerate() {
            if let Some(val) = exec.execute(Some(event)) {
                if i < entry.len() {
                    entry[i] = val;
                }
            }
        }
        *self.timestamp.lock().unwrap() = event.timestamp;
    }

    pub fn clear(&self) {
        self.grouped_values.lock().unwrap().clear();
    }

    pub fn get_grouped_events(&self) -> HashMap<String, StreamEvent> {
        let map = self.grouped_values.lock().unwrap();
        let timestamp = *self.timestamp.lock().unwrap();
        let mut out = HashMap::new();
        for (k, v) in map.iter() {
            let mut se = StreamEvent::new(timestamp, 0, 0, v.len());
            se.output_data = Some(v.clone());
            out.insert(k.clone(), se);
        }
        out
    }

    /// Drain all grouped values returning the timestamp and the values.
    /// This clears the internal storage.
    pub fn drain_grouped_values(&self) -> (i64, HashMap<String, Vec<AttributeValue>>) {
        let mut map = self.grouped_values.lock().unwrap();
        let ts = *self.timestamp.lock().unwrap();
        let out = std::mem::take(&mut *map);
        (ts, out)
    }
}
