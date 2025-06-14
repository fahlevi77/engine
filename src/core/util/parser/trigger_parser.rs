// siddhi_rust/src/core/util/parser/trigger_parser.rs

use std::sync::{Arc, Mutex};

use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::siddhi_app_runtime_builder::SiddhiAppRuntimeBuilder;
use crate::core::stream::stream_junction::StreamJunction;
use crate::core::trigger::TriggerRuntime;
use crate::query_api::constants::TRIGGERED_TIME;
use crate::query_api::definition::{AttributeType, StreamDefinition, TriggerDefinition};

pub struct TriggerParser;

impl TriggerParser {
    pub fn parse(
        builder: &mut SiddhiAppRuntimeBuilder,
        definition: &TriggerDefinition,
        siddhi_app_context: &Arc<SiddhiAppContext>,
    ) -> Result<TriggerRuntime, String> {
        let stream_def = Arc::new(
            StreamDefinition::new(definition.id.clone())
                .attribute(TRIGGERED_TIME.to_string(), AttributeType::LONG),
        );
        builder.add_stream_definition(Arc::clone(&stream_def));
        let junction = Arc::new(Mutex::new(StreamJunction::new(
            definition.id.clone(),
            Arc::clone(&stream_def),
            Arc::clone(siddhi_app_context),
            siddhi_app_context.buffer_size as usize,
            false,
            None,
        )));
        builder.add_stream_junction(definition.id.clone(), Arc::clone(&junction));

        let scheduler = siddhi_app_context.get_scheduler().unwrap_or_else(|| {
            Arc::new(crate::core::util::Scheduler::new(Arc::new(
                crate::core::util::ExecutorService::default(),
            )))
        });
        Ok(TriggerRuntime::new(
            Arc::new(definition.clone()),
            junction,
            scheduler,
        ))
    }
}
