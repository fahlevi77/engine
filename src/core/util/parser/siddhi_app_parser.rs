// siddhi_rust/src/core/util/parser/siddhi_app_parser.rs
// Corresponds to io.siddhi.core.util.parser.SiddhiAppParser
use std::sync::{Arc, Mutex}; // Added Mutex
use std::collections::HashMap; // If QueryParser needs table_map etc. from builder

use crate::query_api::{
    SiddhiApp as ApiSiddhiApp,
    execution::ExecutionElement as ApiExecutionElement,
    definition::{StreamDefinition as ApiStreamDefinition, Attribute as ApiAttribute}, // For fault stream creation
    // Other API definitions will be needed by specific parsers (Table, Window etc.)
};
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::config::siddhi_query_context::SiddhiQueryContext; // QueryParser will need this
use crate::core::siddhi_app_runtime_builder::SiddhiAppRuntimeBuilder;
use crate::core::window::WindowRuntime;
use crate::core::aggregation::AggregationRuntime;
use crate::core::stream::stream_junction::{StreamJunction, OnErrorAction}; // For creating junctions
use crate::query_api::execution::query::input::stream::input_stream::InputStreamTrait;
// use super::query_parser::QueryParser; // To be created or defined in this file for now
// use super::partition_parser::PartitionParser; // To be created
// use super::definition_parser_helpers::*; // For defineStreamDefinitions, defineTableDefinitions etc.
use crate::core::util::SiddhiConstants as CoreSiddhiConstants; // Core constants, if any, vs query_api constants
use super::query_parser::QueryParser; // Use the real QueryParser implementation

// Placeholder for QueryParser and PartitionParser logic
// In a full system, these would be in their own modules.
mod query_parser_placeholder {
    use super::*;
    use crate::query_api::execution::query::Query as ApiQuery;
    use crate::core::query::query_runtime::QueryRuntime;
    pub struct QueryParser;
    impl QueryParser {
        pub fn parse_query(
            _api_query: &ApiQuery,
            _siddhi_app_context: &Arc<SiddhiAppContext>,
            _stream_junction_map: &mut HashMap<String, Arc<Mutex<StreamJunction>>>,
            // _table_map: &HashMap<String, Arc<Mutex<crate::core::siddhi_app_runtime_builder::TableRuntimePlaceholder>>>,
        ) -> Result<QueryRuntime, String> {
            // Simplified: create a dummy QueryRuntime
            let query_name = _api_query.get_query_name_placeholder().unwrap_or_else(|| "unknown_query".to_string());
            let input_stream_id = _api_query.get_input_stream_id_placeholder().unwrap_or_else(|| "unknown_input".to_string());

            let _input_junction = _stream_junction_map.get(&input_stream_id)
                .ok_or_else(|| format!("Input stream '{}' not found for query '{}'", input_stream_id, query_name))?.clone();

            let query_ctx = Arc::new(SiddhiQueryContext::new(
                Arc::clone(_siddhi_app_context),
                query_name.clone(),
                None,
            ));
            Ok(QueryRuntime::new_with_context(query_name, Arc::new(_api_query.clone()), query_ctx))
        }
    }
    // Placeholder getters on query_api::Query
    impl ApiQuery {
        fn get_query_name_placeholder(&self) -> Option<String> { Some("query1".to_string())} // Placeholder
        fn get_input_stream_id_placeholder(&self) -> Option<String> {
            self.input_stream.as_ref().map(|is| is.get_all_stream_ids().join(",")) // Simplified
        }
    }
}


pub struct SiddhiAppParser;

impl SiddhiAppParser {
    // Corresponds to SiddhiAppParser.parse(SiddhiApp siddhiApp, String siddhiAppString, SiddhiContext siddhiContext)
    // The siddhiAppString is already in SiddhiAppContext. SiddhiContext is also in SiddhiAppContext.
    pub fn parse_siddhi_app_runtime_builder(
        api_siddhi_app: &ApiSiddhiApp, // This is from query_api
        siddhi_app_context: Arc<SiddhiAppContext>, // This is from core::config
    ) -> Result<SiddhiAppRuntimeBuilder, String> {

        let mut builder = SiddhiAppRuntimeBuilder::new(siddhi_app_context.clone());

        // Parse @app level annotations to configure defaults
        let mut default_stream_async = false;
        for ann in &api_siddhi_app.annotations {
            if ann.name.eq_ignore_ascii_case("app") {
                for el in &ann.elements {
                    match el.key.to_lowercase().as_str() {
                        "async" => {
                            default_stream_async = el.value.eq_ignore_ascii_case("true");
                        }
                        _ => {}
                    }
                }
            }
        }

        // 1. Define Stream Definitions and create StreamJunctions
        for (stream_id, stream_def_arc) in &api_siddhi_app.stream_definition_map {
            builder.add_stream_definition(Arc::clone(stream_def_arc));

            let mut buffer_size = siddhi_app_context.buffer_size as usize;
            let mut is_async = default_stream_async;
            let mut create_fault_stream = false;

            for ann in &stream_def_arc.abstract_definition.annotations {
                match ann.name.to_lowercase().as_str() {
                    "buffersize" | "buffer.size" => {
                        if let Some(el) = ann.elements.first() {
                            if let Ok(sz) = el.value.parse::<usize>() {
                                buffer_size = sz;
                            }
                        }
                    }
                    "async" => {
                        is_async = true;
                    }
                    "onerror" => {
                        if ann
                            .elements
                            .iter()
                            .find(|e| e.key.eq_ignore_ascii_case("action") && e.value.eq_ignore_ascii_case("stream"))
                            .is_some()
                        {
                            create_fault_stream = true;
                        }
                    }
                    _ => {}
                }
            }

            if create_fault_stream {
                let mut fault_def = ApiStreamDefinition::new(format!("{}{}", crate::query_api::constants::FAULT_STREAM_FLAG, stream_id));
                for attr in &stream_def_arc.abstract_definition.attribute_list {
                    fault_def.abstract_definition.attribute_list.push(attr.clone());
                }
                fault_def.abstract_definition.attribute_list.push(ApiAttribute::new("_error".to_string(), crate::query_api::definition::attribute::Type::OBJECT));
                builder.add_stream_definition(Arc::new(fault_def));
            }

            let stream_junction = Arc::new(Mutex::new(StreamJunction::new(
                stream_id.clone(),
                Arc::clone(stream_def_arc),
                siddhi_app_context.clone(),
                buffer_size,
                is_async,
                None,
            )));
            builder.add_stream_junction(stream_id.clone(), stream_junction);
        }

        // TableDefinitions
        for (table_id, table_def) in &api_siddhi_app.table_definition_map {
            builder.add_table_definition(Arc::clone(table_def));

            let table: Arc<dyn crate::core::table::Table> = if let Some(store_ann) = table_def
                .abstract_definition
                .annotations
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case("store"))
            {
                let mut props = HashMap::new();
                for el in &store_ann.elements {
                    props.insert(el.key.clone(), el.value.clone());
                }
                if let Some(t_type) = props.get("type") {
                    if let Some(factory) = siddhi_app_context
                        .get_siddhi_context()
                        .get_table_factory(t_type)
                    {
                        factory.create(table_id.clone(), props.clone(), siddhi_app_context.get_siddhi_context())?
                    } else if t_type == "jdbc" {
                        let ds = props
                            .get("data_source")
                            .cloned()
                            .unwrap_or_default();
                        Arc::new(crate::core::table::JdbcTable::new(
                            table_id.clone(),
                            ds,
                            siddhi_app_context.get_siddhi_context(),
                        )?)
                    } else {
                        Arc::new(crate::core::table::InMemoryTable::new())
                    }
                } else {
                    Arc::new(crate::core::table::InMemoryTable::new())
                }
            } else {
                Arc::new(crate::core::table::InMemoryTable::new())
            };

            siddhi_app_context
                .get_siddhi_context()
                .add_table(table_id.clone(), table);

            builder.add_table(table_id.clone(), Arc::new(Mutex::new(crate::core::siddhi_app_runtime_builder::TableRuntimePlaceholder::default())));
        }

        // WindowDefinitions
        for (window_id, window_def) in &api_siddhi_app.window_definition_map {
            builder.add_window_definition(Arc::clone(window_def));
            builder.add_window(
                window_id.clone(),
                Arc::new(Mutex::new(WindowRuntime::new(Arc::clone(window_def)))),
            );
        }

        // AggregationDefinitions
        for (agg_id, agg_def) in &api_siddhi_app.aggregation_definition_map {
            builder.add_aggregation_definition(Arc::clone(agg_def));
            builder.add_aggregation_runtime(
                agg_id.clone(),
                Arc::new(Mutex::new(crate::core::aggregation::AggregationRuntime::new(
                    agg_id.clone(),
                    HashMap::new(),
                ))),
            );
        }

        // TODO: Initialize Windows after all tables/windows are defined (as in Java)

        // 2. Parse Execution Elements (Queries, Partitions)
        for exec_element in &api_siddhi_app.execution_element_list {
            match exec_element {
                ApiExecutionElement::Query(api_query) => {
                    // The QueryParser needs access to various maps (stream_junctions, tables, windows, aggregations)
                    // from the builder to resolve references.
                    let query_runtime = QueryParser::parse_query(
                        api_query,
                        &siddhi_app_context,
                        &builder.stream_junction_map,
                        &builder.table_definition_map,
                    )?;
                    builder.add_query_runtime(Arc::new(query_runtime));
                    // TODO: siddhi_app_context.addEternalReferencedHolder(queryRuntime);
                }
                ApiExecutionElement::Partition(api_partition) => {
                    let part_rt = super::partition_parser::PartitionParser::parse(
                        &mut builder,
                        api_partition,
                        &siddhi_app_context,
                    )?;
                    builder.add_partition_runtime(Arc::new(part_rt));
                }
            }
        }

        // TODO: Define Triggers

        Ok(builder)
    }
}

// Helper to get stream_id from ApiInputStream (simplified)
use crate::query_api::execution::query::input::InputStream as ApiInputStream;
impl ApiInputStream {
    fn get_first_stream_id_placeholder(&self) -> Option<String> {
        match self {
            ApiInputStream::Single(s) => Some(s.get_stream_id_str().to_string()),
            ApiInputStream::Join(j) => j.left_input_stream.get_unique_stream_ids().first().cloned(),
            ApiInputStream::State(s) => s.get_unique_stream_ids().first().cloned(),
        }
    }
}
