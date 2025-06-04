pub mod core;
pub mod query_api;
pub mod query_compiler;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::collections::HashMap; // For SiddhiApp's maps (not directly used in this test logic for app construction)

    // query_api module items
    use crate::query_api::siddhi_app::SiddhiApp as ApiSiddhiApp;
    use crate::query_api::definition::{
        StreamDefinition as ApiStreamDefinition,
        Attribute as ApiAttribute,
        attribute::Type as ApiAttributeType // Explicitly importing Type
    };
    use crate::query_api::execution::{
        ExecutionElement as ApiExecutionElement,
        query::Query as ApiQuery,
        query::input::stream::{InputStream as ApiInputStream, SingleInputStream as ApiSingleInputStream},
        query::output::OutputStream as ApiOutputStream,
        query::selection::{Selector as ApiSelector, OutputAttribute as ApiOutputAttribute},
    };
    use crate::query_api::expression::{
        Expression as ApiExpression,
        variable::Variable as ApiVariable,
        constant::{ConstantValueWithFloat as ApiConstantValue, Constant as ApiConstant} // Using renamed ConstantValue
    };
    use crate::query_api::expression::condition::compare::{
        Compare as ApiCompare,
        Operator as ApiCompareOperator
    };

    // core module items
    use crate::core::config::siddhi_context::SiddhiContext;
    // SiddhiAppContext is created inside SiddhiAppRuntime::new currently
    // use crate::core::config::siddhi_app_context::SiddhiAppContext;
    use crate::core::siddhi_app_runtime::SiddhiAppRuntime;
    use crate::core::stream::output::stream_callback::{LogStreamCallback, StreamCallback}; // Assuming LogStreamCallback is here
    use crate::core::event::event::Event as CoreEvent;
    use crate::core::event::value::AttributeValue as CoreAttributeValue;


    #[test]
    fn test_simple_filter_projection_query() {
        println!("Starting test_simple_filter_projection_query...");

        // a. Create SiddhiContext (SiddhiAppContext is created inside SiddhiAppRuntime::new)
        let siddhi_context = Arc::new(SiddhiContext::new());

        // b. Manually Construct query_api::SiddhiApp
        let mut app_to_run = ApiSiddhiApp::new("TestApp".to_string());

        // Input Stream Definition: define stream InputStream (attribute1 int, attribute2 string);
        let input_stream_def = ApiStreamDefinition::new(
            "InputStream".to_string(),
            vec![
                ApiAttribute::new("attribute1".to_string(), ApiAttributeType::INT),
                ApiAttribute::new("attribute2".to_string(), ApiAttributeType::STRING),
            ],
            Vec::new() // No annotations
        );
        app_to_run.stream_definition_map.insert("InputStream".to_string(), Arc::new(input_stream_def));

        // Output Stream Definition: define stream OutputStream (projected_attr1 int, renamed_attr2 string);
        let output_stream_def = ApiStreamDefinition::new(
            "OutputStream".to_string(),
            vec![
                ApiAttribute::new("projected_attr1".to_string(), ApiAttributeType::INT),
                ApiAttribute::new("renamed_attr2".to_string(), ApiAttributeType::STRING),
            ],
            Vec::new()
        );
        app_to_run.stream_definition_map.insert("OutputStream".to_string(), Arc::new(output_stream_def));

        // Query Definition: FROM InputStream[attribute1 > 10] SELECT attribute1 as projected_attr1, attribute2 as renamed_attr2 INSERT INTO OutputStream;

        // Filter: attribute1 > 10
        let filter_condition = ApiExpression::Compare(Box::new(ApiCompare::new(
            Box::new(ApiExpression::Variable(ApiVariable::new("attribute1".to_string()))),
            ApiCompareOperator::GreaterThan,
            Box::new(ApiExpression::Constant(ApiConstant::new(ApiConstantValue::Int(10))))
        )));

        // FROM InputStream[attribute1 > 10]
        let mut api_single_input_stream = ApiSingleInputStream::new_basic_from_id("InputStream".to_string());
        // Add filter using a conceptual method (actual method might be on a builder or direct manipulation)
        // Assuming SingleInputStream from query_api has a method to add filters, or its handlers field is public.
        // For now, this relies on how `QueryParser` would interpret handlers.
        // Let's assume SingleInputStream has a field `pub stream_handlers: Vec<StreamHandler>`
        // and StreamHandler has a Filter variant. This was setup in query_api.
        // The `filter` field was on `SingleInputStreamKind::Basic`.
        // This part of query_api construction needs to be robust.
        // For this test, QueryParser will extract this filter.
        // A more direct way for query_api:
        let mut input_stream_handlers = Vec::new();
        if let Some(condition_expr) = Some(filter_condition) { // Example, filter is not optional for this query
             input_stream_handlers.push(
                 crate::query_api::execution::query::input::handler::StreamHandler::Filter(
                     crate::query_api::execution::query::input::handler::Filter::new(condition_expr)
                 )
             );
        }
        // This assumes SingleInputStream can be constructed with handlers.
        // The current SingleInputStream::new_basic_from_id doesn't take handlers.
        // Let's refine constructor or use a builder pattern for ApiSingleInputStream.
        // For now, manually creating the kind.
        api_single_input_stream.kind = crate::query_api::execution::query::input::stream::SingleInputStreamKind::Basic {
            is_fault_stream: false,
            is_inner_stream: false,
            stream_id: "InputStream".to_string(),
            stream_reference_id: None,
            stream_handlers: input_stream_handlers,
        };
        let input_s = ApiInputStream::Single(api_single_input_stream);


        // Selector: attribute1 as projected_attr1, attribute2 as renamed_attr2
        let selector = ApiSelector::new(vec![
            ApiOutputAttribute::new( // get_expression, get_rename
                Some("projected_attr1".to_string()),
                ApiExpression::Variable(ApiVariable::new("attribute1".to_string()))
            ),
            ApiOutputAttribute::new(
                Some("renamed_attr2".to_string()),
                ApiExpression::Variable(ApiVariable::new("attribute2".to_string()))
            ),
        ]);

        // Output Stream for Query: INSERT INTO OutputStream
        // ApiOutputStream::new takes OutputStreamAction and Option<OutputEventType>
        // Need to create the action struct.
        let insert_action = crate::query_api::execution::query::output::output_stream::InsertIntoStreamAction {
            target_id: "OutputStream".to_string(),
            is_inner_stream: false,
            is_fault_stream: false,
        };
        let output_s = ApiOutputStream::new(
            crate::query_api::execution::query::output::output_stream::OutputStreamAction::InsertInto(insert_action),
            None // Let Query/SiddhiAppRuntimeBuilder determine default OutputEventType
        );

        let query = ApiQuery::new(
            Some(input_s), // input_stream is Option<ApiInputStream>
            selector,      // selector is ApiSelector
            output_s,      // output_stream is ApiOutputStream
            Vec::new()     // annotations
        );
        app_to_run.execution_element_list.push(ApiExecutionElement::Query(query));

        let runnable_api_app = Arc::new(app_to_run);

        // c. Create SiddhiAppRuntime
        // SiddhiAppRuntime::new now takes Arc<ApiSiddhiApp> and Arc<SiddhiContext>
        println!("Test: Creating SiddhiAppRuntime...");
        let runtime = SiddhiAppRuntime::new(runnable_api_app, siddhi_context)
            .expect("Test: Failed to create SiddhiAppRuntime");
        println!("Test: SiddhiAppRuntime created successfully.");

        // d. Add LogStreamCallback
        let callback = Box::new(LogStreamCallback::new("OutputStream".to_string()));
        runtime.add_callback("OutputStream", callback)
            .expect("Test: Failed to add callback to OutputStream");
        println!("Test: Callback added to OutputStream.");

        // e. Start SiddhiAppRuntime
        runtime.start();
        println!("Test: SiddhiAppRuntime started.");

        // f. Get InputHandler
        let input_handler = runtime.get_input_handler("InputStream")
            .expect("Test: Failed to get InputHandler for InputStream");
        println!("Test: InputHandler for InputStream obtained.");

        // g. Send Events
        println!("Test: Sending events...");
        let event1_ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
        let event1_data = vec![
            CoreAttributeValue::Int(20),
            CoreAttributeValue::String("event_one_val2".to_string()),
        ];
        input_handler.send_event_with_timestamp(event1_ts, event1_data).expect("Test: Failed to send event1");
        println!("Test: Sent event1 (attribute1=20, should pass filter)");

        let event2_ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
        let event2_data = vec![
            CoreAttributeValue::Int(5),
            CoreAttributeValue::String("event_two_val2".to_string()),
        ];
        input_handler.send_event_with_timestamp(event2_ts, event2_data).expect("Test: Failed to send event2");
        println!("Test: Sent event2 (attribute1=5, should be filtered out)");

        let event3_ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
        let event3_data = vec![
            CoreAttributeValue::Int(30),
            CoreAttributeValue::String("event_three_val2".to_string()),
        ];
        input_handler.send_event_with_timestamp(event3_ts, event3_data).expect("Test: Failed to send event3");
        println!("Test: Sent event3 (attribute1=30, should pass filter)");

        // Allow some time for async processing if any (though current setup is mostly sync)
        // std::thread::sleep(std::time::Duration::from_millis(100));

        // h. Shutdown SiddhiAppRuntime
        runtime.shutdown();
        println!("Test: SiddhiAppRuntime shutdown.");
        println!("Test: test_simple_filter_projection_query finished. Check console output for LogStreamCallback.");

        // i. Assertions (Manual Observation of Console Output)
        // Expected: Event 1 and Event 3 should appear in output, Event 2 should not.
        // Output should have two attributes: projected_attr1 (int) and renamed_attr2 (string).
    }
}
