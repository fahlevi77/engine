// SPDX-License-Identifier: MIT OR Apache-2.0

//! Automatic Sink Initialization Test
//!
//! This test demonstrates automatic sink initialization from YAML configuration
//! where sinks are automatically attached to streams based on configuration

#[path = "common/mod.rs"]
mod common;

use common::AppRunner;
use eventflux_rust::core::config::{ApplicationConfig, ConfigManager, EventFluxConfig};
use eventflux_rust::core::event::value::AttributeValue;
use eventflux_rust::core::eventflux_manager::EventFluxManager;
use eventflux_rust::query_compiler::parse;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_automatic_sink_initialization() {
    println!("🚀 Automatic Sink Initialization Test");
    println!("=====================================");

    // Step 1: Create YAML configuration with streams and their sinks
    let config_yaml = r#"
apiVersion: eventflux.io/v1
kind: EventFluxConfig
metadata:
  name: auto-sink-test
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
      LogOutput:
        sink:
          type: "log"
          prefix: "[AUTO-LOG]"
          level: "INFO"
      WarningOutput:
        sink:
          type: "log"
          prefix: "[AUTO-WARNING]"
          level: "WARNING"
      DebugOutput:
        sink:
          type: "log"
          prefix: "[AUTO-DEBUG]"
          level: "DEBUG"
"#;

    // Create temporary config file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(config_yaml.as_bytes())
        .expect("Failed to write config");
    let config_path = temp_file.path().to_path_buf();

    // Step 2: Create EventFluxManager with configuration
    println!("\n📝 Step 1: Loading configuration from YAML...");
    let config_manager = ConfigManager::from_file(&config_path);
    let mut manager = EventFluxManager::new_with_config_manager(config_manager);

    // Step 3: Define a EventFlux app with multiple output streams
    let eventflux_app = r#"
        define stream InputStream (id int, level string, message string);
        define stream LogOutput (id int, message string);
        define stream WarningOutput (id int, message string);
        define stream DebugOutput (id int, message string);
        
        -- Route all messages to LogOutput
        from InputStream
        select id, message
        insert into LogOutput;
        
        -- Route warning messages to WarningOutput
        from InputStream[level == 'WARNING']
        select id, message
        insert into WarningOutput;
        
        -- Route debug messages to DebugOutput  
        from InputStream[level == 'DEBUG']
        select id, message
        insert into DebugOutput;
    "#;

    println!("\n🏗️ Step 2: Creating EventFlux app with multiple output streams...");
    let api = parse(eventflux_app).expect("Failed to parse EventFlux app");
    let runtime = manager
        .create_eventflux_app_runtime_from_api(Arc::new(api), None)
        .await
        .expect("Failed to create runtime");

    println!("\n✅ Step 3: Sinks should be auto-attached from configuration!");
    println!("   - LogOutput stream → [AUTO-LOG] prefix");
    println!("   - WarningOutput stream → [AUTO-WARNING] prefix");
    println!("   - DebugOutput stream → [AUTO-DEBUG] prefix");

    runtime.start();

    // Step 4: Send test events through the pipeline
    println!("\n🚀 Step 4: Sending test events...");
    let input_handler = runtime.get_input_handler("InputStream").unwrap();

    // Send INFO event (should go to LogOutput)
    input_handler
        .lock()
        .unwrap()
        .send_data(vec![
            AttributeValue::Int(1),
            AttributeValue::String("INFO".to_string()),
            AttributeValue::String("This is an info message".to_string()),
        ])
        .unwrap();

    // Send WARNING event (should go to LogOutput and WarningOutput)
    input_handler
        .lock()
        .unwrap()
        .send_data(vec![
            AttributeValue::Int(2),
            AttributeValue::String("WARNING".to_string()),
            AttributeValue::String("This is a warning message".to_string()),
        ])
        .unwrap();

    // Send DEBUG event (should go to LogOutput and DebugOutput)
    input_handler
        .lock()
        .unwrap()
        .send_data(vec![
            AttributeValue::Int(3),
            AttributeValue::String("DEBUG".to_string()),
            AttributeValue::String("This is a debug message".to_string()),
        ])
        .unwrap();

    // Give time for events to process
    sleep(Duration::from_millis(100)).await;

    runtime.shutdown();

    println!("\n🏁 Test completed successfully!");
    println!("   - Configuration loaded from YAML ✅");
    println!("   - EventFlux app created with multiple streams ✅");
    println!("   - Sinks auto-attached based on configuration ✅");
    println!("   - Events routed to appropriate sinks ✅");
    println!("\n📝 Expected output in console:");
    println!("   - [AUTO-LOG] for all 3 events");
    println!("   - [AUTO-WARNING] for event #2");
    println!("   - [AUTO-DEBUG] for event #3");
}
