use crate::core::event::complex_event::ComplexEvent;
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::util::siddhi_constants::KEY_DELIMITER;

/// Generates a composite key for group-by operations using a list of expression executors.
#[derive(Debug)]
pub struct GroupByKeyGenerator {
    executors: Vec<Box<dyn ExpressionExecutor>>,
}

impl Clone for GroupByKeyGenerator {
    fn clone(&self) -> Self {
        Self {
            executors: Vec::new(),
        }
    }
}

impl GroupByKeyGenerator {
    pub fn new(executors: Vec<Box<dyn ExpressionExecutor>>) -> Self {
        Self { executors }
    }

    pub fn construct_event_key(&self, event: &dyn ComplexEvent) -> Option<String> {
        if self.executors.is_empty() {
            return None;
        }
        let mut parts = Vec::with_capacity(self.executors.len());
        for exec in &self.executors {
            match exec.execute(Some(event)) {
                Some(val) => parts.push(val.to_string()),
                None => parts.push("null".to_string()),
            }
        }
        Some(parts.join(KEY_DELIMITER))
    }
}
