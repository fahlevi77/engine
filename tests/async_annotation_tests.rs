// Tests for @Async annotation parsing and stream creation
use siddhi_rust::core::config::siddhi_context::SiddhiContext;
use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::query_api::definition::attribute::Type as AttributeType;

#[tokio::test]
async fn test_async_annotation_basic() {
    let mut manager = SiddhiManager::new();

    let siddhi_app_string = r#"
        @Async(buffer_size='1024', workers='2', batch_size_max='10')
        define stream StockStream (symbol string, price float, volume long);
    "#;

    let result = manager.create_siddhi_app_runtime_from_string(siddhi_app_string).await;
    assert!(
        result.is_ok(),
        "Failed to parse @Async annotation: {:?}",
        result.as_ref().err()
    );

    let app_runtime = result.unwrap();
    let stream_definitions = &app_runtime.siddhi_app.stream_definition_map;

    assert!(stream_definitions.contains_key("StockStream"));
    let stream_def = stream_definitions.get("StockStream").unwrap();

    // Check that the stream has the @Async annotation
    let async_annotation = stream_def
        .abstract_definition
        .annotations
        .iter()
        .find(|ann| ann.name.eq_ignore_ascii_case("async"));

    assert!(
        async_annotation.is_some(),
        "Stream should have @Async annotation"
    );

    let async_ann = async_annotation.unwrap();

    // Check annotation parameters
    let buffer_size_element = async_ann
        .elements
        .iter()
        .find(|el| el.key.eq_ignore_ascii_case("buffer_size"));
    assert!(buffer_size_element.is_some());
    assert_eq!(buffer_size_element.unwrap().value, "1024");

    let workers_element = async_ann
        .elements
        .iter()
        .find(|el| el.key.eq_ignore_ascii_case("workers"));
    assert!(workers_element.is_some());
    assert_eq!(workers_element.unwrap().value, "2");

    let batch_size_element = async_ann
        .elements
        .iter()
        .find(|el| el.key.eq_ignore_ascii_case("batch_size_max"));
    assert!(batch_size_element.is_some());
    assert_eq!(batch_size_element.unwrap().value, "10");
}

#[tokio::test]
async fn test_async_annotation_minimal() {
    let mut manager = SiddhiManager::new();

    let siddhi_app_string = r#"
        @Async
        define stream MinimalAsyncStream (id int, value string);
    "#;

    let result = manager.create_siddhi_app_runtime_from_string(siddhi_app_string).await;
    assert!(
        result.is_ok(),
        "Failed to parse minimal @Async annotation: {:?}",
        result.as_ref().err()
    );

    let app_runtime = result.unwrap();
    let stream_definitions = &app_runtime.siddhi_app.stream_definition_map;

    assert!(stream_definitions.contains_key("MinimalAsyncStream"));
    let stream_def = stream_definitions.get("MinimalAsyncStream").unwrap();

    // Check that the stream has the @Async annotation
    let async_annotation = stream_def
        .abstract_definition
        .annotations
        .iter()
        .find(|ann| ann.name.eq_ignore_ascii_case("async"));

    assert!(
        async_annotation.is_some(),
        "Stream should have @Async annotation"
    );
}

#[tokio::test]
async fn test_config_annotation_async() {
    let mut manager = SiddhiManager::new();

    let siddhi_app_string = r#"
        @config(async='true')
        define stream ConfigAsyncStream (symbol string, price float);
    "#;

    let result = manager.create_siddhi_app_runtime_from_string(siddhi_app_string).await;
    assert!(
        result.is_ok(),
        "Failed to parse @config annotation: {:?}",
        result.as_ref().err()
    );

    let app_runtime = result.unwrap();
    let stream_definitions = &app_runtime.siddhi_app.stream_definition_map;

    assert!(stream_definitions.contains_key("ConfigAsyncStream"));
    let stream_def = stream_definitions.get("ConfigAsyncStream").unwrap();

    // Check that the stream has the @config annotation
    let config_annotation = stream_def
        .abstract_definition
        .annotations
        .iter()
        .find(|ann| ann.name.eq_ignore_ascii_case("config"));

    assert!(
        config_annotation.is_some(),
        "Stream should have @config annotation"
    );

    let config_ann = config_annotation.unwrap();
    let async_element = config_ann
        .elements
        .iter()
        .find(|el| el.key.eq_ignore_ascii_case("async"));
    assert!(async_element.is_some());
    assert_eq!(async_element.unwrap().value, "true");
}

#[tokio::test]
async fn test_app_level_async_annotation() {
    let mut manager = SiddhiManager::new();

    let siddhi_app_string = r#"
        @app(async='true')
        
        define stream AutoAsyncStream (id int, value string);
        
        define stream RegularStream (name string, count int);
    "#;

    let result = manager.create_siddhi_app_runtime_from_string(siddhi_app_string).await;
    assert!(
        result.is_ok(),
        "Failed to parse app-level @app annotation: {:?}",
        result.as_ref().err()
    );

    let app_runtime = result.unwrap();

    // Both streams should exist
    let stream_definitions = &app_runtime.siddhi_app.stream_definition_map;
    assert!(stream_definitions.contains_key("AutoAsyncStream"));
    assert!(stream_definitions.contains_key("RegularStream"));
}

#[tokio::test]
async fn test_multiple_async_streams() {
    let mut manager = SiddhiManager::new();

    let siddhi_app_string = r#"
        @Async(buffer_size='512')
        define stream Stream1 (id int);
        
        @Async(buffer_size='2048', workers='4')
        define stream Stream2 (name string);
        
        define stream Stream3 (value float);
    "#;

    let result = manager.create_siddhi_app_runtime_from_string(siddhi_app_string).await;
    assert!(
        result.is_ok(),
        "Failed to parse multiple async streams: {:?}",
        result.as_ref().err()
    );

    let app_runtime = result.unwrap();
    let stream_definitions = &app_runtime.siddhi_app.stream_definition_map;

    // Check Stream1 has correct buffer size
    let stream1 = stream_definitions.get("Stream1").unwrap();
    let async_ann1 = stream1
        .abstract_definition
        .annotations
        .iter()
        .find(|ann| ann.name.eq_ignore_ascii_case("async"))
        .unwrap();
    let buffer_size1 = async_ann1
        .elements
        .iter()
        .find(|el| el.key.eq_ignore_ascii_case("buffer_size"))
        .unwrap();
    assert_eq!(buffer_size1.value, "512");

    // Check Stream2 has correct parameters
    let stream2 = stream_definitions.get("Stream2").unwrap();
    let async_ann2 = stream2
        .abstract_definition
        .annotations
        .iter()
        .find(|ann| ann.name.eq_ignore_ascii_case("async"))
        .unwrap();
    let buffer_size2 = async_ann2
        .elements
        .iter()
        .find(|el| el.key.eq_ignore_ascii_case("buffer_size"))
        .unwrap();
    assert_eq!(buffer_size2.value, "2048");
    let workers2 = async_ann2
        .elements
        .iter()
        .find(|el| el.key.eq_ignore_ascii_case("workers"))
        .unwrap();
    assert_eq!(workers2.value, "4");

    // Check Stream3 has no async annotation
    let stream3 = stream_definitions.get("Stream3").unwrap();
    let async_ann3 = stream3
        .abstract_definition
        .annotations
        .iter()
        .find(|ann| ann.name.eq_ignore_ascii_case("async"));
    assert!(
        async_ann3.is_none(),
        "Stream3 should not have @Async annotation"
    );
}

#[tokio::test]
async fn test_async_annotation_with_query() {
    let mut manager = SiddhiManager::new();

    let siddhi_app_string = r#"
        @Async(buffer_size='1024')
        define stream InputStream (symbol string, price float, volume long);
        
        define stream OutputStream (symbol string, avgPrice float);
        
        from InputStream#window:time(10 sec)
        select symbol, avg(price) as avgPrice
        insert into OutputStream;
    "#;

    let result = manager.create_siddhi_app_runtime_from_string(siddhi_app_string).await;
    assert!(
        result.is_ok(),
        "Failed to parse async stream with query: {:?}",
        result.as_ref().err()
    );

    let app_runtime = result.unwrap();
    let stream_definitions = &app_runtime.siddhi_app.stream_definition_map;

    assert!(stream_definitions.contains_key("InputStream"));
    assert!(stream_definitions.contains_key("OutputStream"));

    // InputStream should have @Async annotation
    let input_stream = stream_definitions.get("InputStream").unwrap();
    let async_annotation = input_stream
        .abstract_definition
        .annotations
        .iter()
        .find(|ann| ann.name.eq_ignore_ascii_case("async"));
    assert!(async_annotation.is_some());

    // OutputStream should not have @Async annotation
    let output_stream = stream_definitions.get("OutputStream").unwrap();
    let no_async_annotation = output_stream
        .abstract_definition
        .annotations
        .iter()
        .find(|ann| ann.name.eq_ignore_ascii_case("async"));
    assert!(no_async_annotation.is_none());
}
