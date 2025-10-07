// SPDX-License-Identifier: MIT OR Apache-2.0

//! Verification test to check if automatic sink initialization is working

#[path = "common/mod.rs"]
mod common;

use eventflux_rust::core::config::{ApplicationConfig, ConfigManager, EventFluxConfig};
use eventflux_rust::core::event::value::AttributeValue;
use eventflux_rust::core::eventflux_manager::EventFluxManager;
use eventflux_rust::query_compiler::parse;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_verify_auto_sink_attachment() {
    println!("🔍 Verification: Auto Sink Attachment Test");
    println!("===========================================");

    let config_yaml = r#"
apiVersion: eventflux.io/v1
kind: EventFluxConfig
metadata:
  name: verify-test
eventflux:
  runtime:
    mode: single-node
    performance:
      thread_pool_size: 4
      event_buffer_size: 1024
      batch_processing: true
      async_processing: false
applications:
  test-app:
    streams:
      TestOutput:
        sink:
          type: "log"
          prefix: "[VERIFICATION-TEST]"
          level: "INFO"
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(config_yaml.as_bytes())
        .expect("Failed to write config");
    let config_path = temp_file.path().to_path_buf();

    println!("\n📝 Loading configuration...");
    let config_manager = ConfigManager::from_file(&config_path);
    let mut manager = EventFluxManager::new_with_config_manager(config_manager);

    let eventflux_app = r#"
        define stream Input (id int, message string);
        define stream TestOutput (id int, message string);
        
        from Input
        select id, message
        insert into TestOutput;
    "#;

    println!("🏗️ Creating EventFlux app...");
    let api = parse(eventflux_app).expect("Failed to parse EventFlux app");
    let runtime = manager
        .create_eventflux_app_runtime_from_api(Arc::new(api), None)
        .await
        .expect("Failed to create runtime");

    runtime.start();

    println!("🚀 Sending test event...");
    let input_handler = runtime.get_input_handler("Input").unwrap();

    input_handler
        .lock()
        .unwrap()
        .send_data(vec![
            AttributeValue::Int(42),
            AttributeValue::String("Verification test message".to_string()),
        ])
        .unwrap();

    sleep(Duration::from_millis(100)).await;
    runtime.shutdown();

    println!("\n✅ Test completed - check console for [VERIFICATION-TEST] message");
}

#[tokio::test]
async fn test_verify_configuration_structure() {
    println!("🔍 Verification: Configuration Structure Test");
    println!("==============================================");

    let config_yaml = r#"
apiVersion: eventflux.io/v1
kind: EventFluxConfig
metadata:
  name: structure-test
eventflux:
  runtime:
    mode: single-node
    performance:
      thread_pool_size: 4
      event_buffer_size: 1024
      batch_processing: true
      async_processing: false
applications:
  test-app:
    streams:
      TestStream:
        sink:
          type: "log"
          prefix: "[CONFIG-STRUCTURE-TEST]"
          level: "DEBUG"
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(config_yaml.as_bytes())
        .expect("Failed to write config");
    let config_path = temp_file.path().to_path_buf();

    println!("\n📝 Loading and parsing configuration...");
    let config_manager = ConfigManager::from_file(&config_path);
    let config = config_manager
        .load_unified_config()
        .await
        .expect("Failed to load config");

    println!("✅ Configuration parsed successfully");
    println!("📊 Number of applications: {}", config.applications.len());

    if let Some(app_config) = config.applications.get("test-app") {
        println!(
            "📊 Number of streams in test-app: {}",
            app_config.streams.len()
        );

        if let Some(stream_config) = app_config.streams.get("TestStream") {
            println!("✅ TestStream found in configuration");
            if let Some(ref sink_config) = stream_config.sink {
                println!("✅ Sink configuration found for TestStream");
                println!("📋 Sink type: {}", sink_config.sink_type);
                println!("📋 Sink connection config: {:?}", sink_config.connection);
            } else {
                println!("❌ No sink configuration found for TestStream");
            }
        } else {
            println!("❌ TestStream not found in configuration");
        }
    } else {
        println!("❌ test-app not found in configuration");
    }
}
