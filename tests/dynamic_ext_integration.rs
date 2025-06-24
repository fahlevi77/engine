use std::sync::Arc;
use siddhi_rust::core::config::{siddhi_app_context::SiddhiAppContext, siddhi_query_context::SiddhiQueryContext};
use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::core::util::parser::{ExpressionParserContext, parse_expression};
use siddhi_rust::query_api::definition::attribute::Type as AttrType;
use siddhi_rust::query_api::definition::StreamDefinition;
use siddhi_rust::query_api::expression::Expression;
use siddhi_rust::query_api::siddhi_app::SiddhiApp;

#[test]
fn test_dynamic_extension_loading() {
    let manager = SiddhiManager::new();
    let lib_path = custom_dyn_ext::library_path();
    manager.set_extension("dynlib", lib_path.to_str().unwrap().to_string()).unwrap();

    // verify window factory registered
    let ctx = manager.siddhi_context();
    assert!(ctx.get_window_factory("dynWindow").is_some());
    assert!(ctx.get_attribute_aggregator_factory("dynConstAgg").is_some());

    // Use custom aggregator via expression parser
    let app = Arc::new(SiddhiApp::new("app".to_string()));
    let app_ctx = Arc::new(SiddhiAppContext::new(ctx, "app".to_string(), Arc::clone(&app), String::new()));
    let q_ctx = Arc::new(SiddhiQueryContext::new(Arc::clone(&app_ctx), "q".to_string(), None));
    let stream_def = Arc::new(StreamDefinition::new("S".to_string()).attribute("v".to_string(), AttrType::INT));
    let meta = siddhi_rust::core::event::stream::meta_stream_event::MetaStreamEvent::new_for_single_input(Arc::clone(&stream_def));
    let mut map = std::collections::HashMap::new();
    map.insert("S".to_string(), Arc::new(meta));
    let parse_ctx = ExpressionParserContext {
        siddhi_app_context: app_ctx,
        siddhi_query_context: q_ctx,
        stream_meta_map: map,
        table_meta_map: std::collections::HashMap::new(),
        window_meta_map: std::collections::HashMap::new(),
        aggregation_meta_map: std::collections::HashMap::new(),
        state_meta_map: std::collections::HashMap::new(),
        stream_positions: {
            let mut m = std::collections::HashMap::new();
            m.insert("S".to_string(), 0);
            m
        },
        default_source: "S".to_string(),
        query_name: "Q",
    };
    let expr = Expression::function_no_ns("dynConstAgg".to_string(), vec![]);
    let exec = parse_expression(&expr, &parse_ctx).unwrap();
    assert_eq!(exec.execute(None), Some(siddhi_rust::core::event::value::AttributeValue::Int(7)));
}
