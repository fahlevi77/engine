// siddhi_rust/src/core/util/parser/query_parser.rs
use crate::query_api::{
    execution::query::Query as ApiQuery,
    execution::query::input::InputStream as ApiInputStream,
    definition::StreamDefinition as ApiStreamDefinition,
    definition::Attribute as ApiAttribute, // For constructing output attributes
    // Import other query_api parts as needed (Selector, OutputAttribute, OutputStream)
    execution::query::selection::OutputAttribute as ApiOutputAttribute,
    execution::query::output::OutputStream as ApiOutputStream,
    execution::query::output::OutputEventType as ApiOutputEventType,
    expression::Expression as ApiExpression, // Added this import
};
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::config::siddhi_query_context::SiddhiQueryContext;
use crate::core::stream::stream_junction::StreamJunction;
use crate::core::query::query_runtime::QueryRuntime;
use crate::core::query::processor::Processor; // Trait
use crate::core::query::processor::stream::filter::FilterProcessor;
use crate::core::query::selector::select_processor::SelectProcessor;
use crate::core::query::selector::attribute::OutputAttributeProcessor; // OAP
use crate::core::query::output::insert_into_stream_processor::InsertIntoStreamProcessor;
use super::expression_parser::{parse_expression, ExpressionParserContext};
use crate::core::event::stream::meta_stream_event::MetaStreamEvent;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
// use rand; // For generating random names, if not using query name from annotation

pub struct QueryParser;

impl QueryParser {
    pub fn parse_query(
        api_query: &ApiQuery,
        siddhi_app_context: &Arc<SiddhiAppContext>,
        // These maps are from SiddhiAppRuntimeBuilder
        stream_junction_map: &HashMap<String, Arc<Mutex<StreamJunction>>>,
        // table_map: &HashMap<String, Arc<Mutex<crate::core::siddhi_app_runtime_builder::TableRuntimePlaceholder>>>,
        // window_map: &HashMap<String, Arc<Mutex<crate::core::siddhi_app_runtime_builder::WindowRuntimePlaceholder>>>,
        // aggregation_map: &HashMap<String, Arc<Mutex<crate::core::siddhi_app_runtime_builder::AggregationRuntimePlaceholder>>>,
    ) -> Result<QueryRuntime, String> {

        // 1. Determine Query Name (from @info(name='foo') or generate)
        let query_name = api_query.annotations.iter()
            .find(|ann| ann.name == "info")
            .and_then(|ann| ann.elements.iter().find(|el| el.key == "name"))
            .map(|el| el.value.clone())
            .unwrap_or_else(|| format!("query_{}", uuid::Uuid::new_v4().hyphenated()));


        let siddhi_query_context = Arc::new(SiddhiQueryContext::new(
            Arc::clone(siddhi_app_context),
            query_name.clone(),
            None // partition_id, assuming not in a partition for now
        ));

        // 2. Identify input stream & get its junction
        let input_stream_api = api_query.input_stream.as_ref()
            .ok_or_else(|| format!("Query '{}' has no input stream defined.", query_name))?;

        // This simplified parser currently only supports SingleInputStream as input.
        let (input_stream_id, api_single_input_stream) = match input_stream_api {
            ApiInputStream::Single(single_in_stream) => {
                (single_in_stream.get_stream_id_str().to_string(), single_in_stream)
            }
            _ => return Err(format!("Query '{}': Only SingleInputStream supported for now. Found {:?}",
                                    query_name, input_stream_api)),
        };

        let input_junction = stream_junction_map.get(&input_stream_id)
            .ok_or_else(|| format!("Input stream '{}' not found for query '{}'", input_stream_id, query_name))?
            .clone();

        let input_stream_def_from_junction = input_junction.lock().expect("Input junction Mutex poisoned").get_stream_definition();

        // 3. Create MetaStreamEvent for ExpressionParserContext (based on single input stream)
        let meta_input_event = Arc::new(MetaStreamEvent::new_for_single_input(input_stream_def_from_junction));
        let expr_parser_context = ExpressionParserContext {
            siddhi_app_context: Arc::clone(siddhi_app_context),
            meta_input_event: Arc::clone(&meta_input_event),
            query_name: &query_name, // Pass as reference
        };

        let mut processor_chain_head: Option<Arc<Mutex<dyn Processor>>> = None;
        let mut last_processor_in_chain: Option<Arc<Mutex<dyn Processor>>> = None;

        // Helper to link processors in a chain
        let mut link_processor = |new_processor_arc: Arc<Mutex<dyn Processor>>| {
            if processor_chain_head.is_none() {
                processor_chain_head = Some(new_processor_arc.clone());
            }
            if let Some(ref last_p_arc) = last_processor_in_chain {
                last_p_arc.lock().expect("Processor Mutex poisoned").set_next_processor(Some(new_processor_arc.clone()));
            }
            last_processor_in_chain = Some(new_processor_arc);
        };

        // 4. Filter (Optional) - from SingleInputStream's handlers
        // TODO: Iterate through all handlers on api_single_input_stream, not just one filter.
        // For now, assuming only one optional filter for simplicity.
        if let Some(filter_api_expr) = api_single_input_stream.get_stream_handlers().iter()
            .find_map(|h| match h { crate::query_api::execution::query::input::handler::StreamHandler::Filter(f) => Some(&f.filter_expression), _ => None }) {
            let condition_executor = parse_expression(filter_api_expr, &expr_parser_context)?;
            let filter_processor = Arc::new(Mutex::new(FilterProcessor::new(
                condition_executor,
                Arc::clone(siddhi_app_context),
                Arc::clone(&siddhi_query_context),
                // query_name.clone() // query_name is in siddhi_query_context
            )?)); // FilterProcessor::new now returns Result
            link_processor(filter_processor);
        }

        // 5. Selector (Projections)
        let api_selector = &api_query.selector; // Selector is not Option in query_api::Query
        let mut oaps = Vec::new();
        let mut output_attributes_for_def = Vec::new();

        for (idx, api_out_attr) in api_selector.selection_list.iter().enumerate() {
            let expr_exec = parse_expression(&api_out_attr.expression, &expr_parser_context)?;
            // OutputAttributeProcessor needs the output position.
            let oap = OutputAttributeProcessor::new(expr_exec, idx);

            let attr_name = api_out_attr.rename.clone().unwrap_or_else(|| {
                // Try to infer name from expression, or generate. Very simplified.
                // Java's OutputAttribute(Variable) uses variable.getAttributeName().
                // If expression is a Variable, use its name.
                if let ApiExpression::Variable(v) = &api_out_attr.expression {
                    v.attribute_name.clone()
                } else {
                    format!("_col_{}", idx)
                }
            });
            output_attributes_for_def.push(ApiAttribute::new(attr_name, oap.get_return_type()));
            oaps.push(oap);
        }

        // Determine the ID for the stream produced by this SelectProcessor
        let output_stream_id_from_query = api_query.output_stream.get_target_id() // output_stream is not Option
            .ok_or_else(|| format!("Query '{}' must have a target output stream for INSERT INTO", query_name))?;

        let select_output_stream_def = Arc::new(ApiStreamDefinition::new(
            format!("_internal_{}_select_output", query_name),
            output_attributes_for_def,
            Vec::new()
        ));

        let select_processor = Arc::new(Mutex::new(SelectProcessor::new(
            api_selector, // Pass the API selector for limit/offset etc.
            true, true, // current_on, expired_on defaults for now
            Arc::clone(siddhi_app_context),
            Arc::clone(&siddhi_query_context),
            oaps,
            select_output_stream_def,
            None, None, None, None, // Placeholders for having, groupby, orderby, batching
        )));
        link_processor(select_processor);

        // 6. Output Processor (e.g., InsertIntoStreamProcessor)
        // This needs to match on api_query.output_stream.action
        match &api_query.output_stream.action {
            crate::query_api::execution::query::output::output_stream::OutputStreamAction::InsertInto(insert_action) => {
                 let target_junction = stream_junction_map.get(&insert_action.target_id)
                    .ok_or_else(|| format!("Output stream '{}' not found for query '{}'", insert_action.target_id, query_name))?
                    .clone();

                let insert_processor = Arc::new(Mutex::new(InsertIntoStreamProcessor::new(
                    target_junction,
                    Arc::clone(siddhi_app_context),
                    Arc::clone(&siddhi_query_context),
                )));
                link_processor(insert_processor);
            }
            _ => return Err(format!("Query '{}': Only INSERT INTO output supported for now.", query_name)),
        }

        // 7. Create QueryRuntime
        let mut query_runtime = QueryRuntime::new(query_name.clone() /*, siddhi_query_context */);
        query_runtime.processor_chain_head = processor_chain_head;

        // 8. Register the entry processor with the input stream junction
        if let Some(head_proc_arc) = &query_runtime.processor_chain_head {
            input_junction.lock().expect("Input junction Mutex poisoned").add_subscriber(Arc::clone(head_proc_arc));
        } else {
            // This might be valid if a query only defines things but has no direct input-to-output flow
            // (e.g. a query that only defines named windows used by other queries).
            // For now, assume a query must have a processor chain if it has an input stream.
            return Err(format!("Query '{}' resulted in an empty processor chain despite having an input stream.", query_name));
        }

        Ok(query_runtime)
    }
}
