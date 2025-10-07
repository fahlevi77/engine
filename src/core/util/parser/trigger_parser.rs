// eventflux_rust/src/core/util/parser/trigger_parser.rs

use std::sync::{Arc, Mutex};

use crate::core::config::eventflux_app_context::EventFluxAppContext;
use crate::core::eventflux_app_runtime_builder::EventFluxAppRuntimeBuilder;
use crate::core::stream::stream_junction::StreamJunction;
use crate::core::trigger::TriggerRuntime;
use crate::query_api::constants::TRIGGERED_TIME;
use crate::query_api::definition::{AttributeType, StreamDefinition, TriggerDefinition};

pub struct TriggerParser;

impl TriggerParser {
    pub fn parse(
        builder: &mut EventFluxAppRuntimeBuilder,
        definition: &TriggerDefinition,
        eventflux_app_context: &Arc<EventFluxAppContext>,
    ) -> Result<TriggerRuntime, String> {
        let stream_def = Arc::new(
            StreamDefinition::new(definition.id.clone())
                .attribute(TRIGGERED_TIME.to_string(), AttributeType::LONG),
        );
        builder.add_stream_definition(Arc::clone(&stream_def));
        let junction = Arc::new(Mutex::new(StreamJunction::new(
            definition.id.clone(),
            Arc::clone(&stream_def),
            Arc::clone(eventflux_app_context),
            eventflux_app_context.buffer_size as usize,
            false,
            None,
        )));
        builder.add_stream_junction(definition.id.clone(), Arc::clone(&junction));

        let scheduler = eventflux_app_context.get_scheduler().unwrap_or_else(|| {
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
