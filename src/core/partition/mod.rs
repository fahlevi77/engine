use std::sync::Arc;

use crate::core::query::query_runtime::{QueryRuntime, QueryRuntimeTrait};

/// Runtime representation of a Partition.
#[derive(Debug, Default)]
pub struct PartitionRuntime {
    pub query_runtimes: Vec<Arc<QueryRuntime>>,
}

impl PartitionRuntime {
    pub fn new() -> Self {
        Self {
            query_runtimes: Vec::new(),
        }
    }

    pub fn add_query_runtime(&mut self, qr: Arc<QueryRuntime>) {
        self.query_runtimes.push(qr);
    }

    pub fn start(&self) {
        for qr in &self.query_runtimes {
            println!("Starting query runtime {} in partition", qr.get_query_id());
        }
    }

    pub fn shutdown(&self) {
        for qr in &self.query_runtimes {
            println!("Stopping query runtime {} in partition", qr.get_query_id());
        }
    }
}

pub mod parser;
pub use parser::PartitionParser;
