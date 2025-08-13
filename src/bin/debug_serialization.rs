use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Minimal test to reproduce the serialization hang
#[derive(Debug, Clone, Serialize, Deserialize)]
enum TestSerializableAttributeValue {
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Object {
        type_name: String,
        is_some: bool,
        serialized_data: Option<String>,
    },
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestSerializableStreamEvent {
    timestamp: i64,
    before_window_data: Vec<TestSerializableAttributeValue>,
    output_data: Option<Vec<TestSerializableAttributeValue>>,
    event_type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestSerializableSessionChunk {
    events: Vec<TestSerializableStreamEvent>,
    start_timestamp: i64,
    end_timestamp: i64,
    alive_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestSerializableSessionContainer {
    current_session: TestSerializableSessionChunk,
    previous_session: TestSerializableSessionChunk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestSerializableSessionState {
    session_map: HashMap<String, TestSerializableSessionContainer>,
    expired_event_chunk: TestSerializableSessionChunk,
}

fn main() {
    println!("Creating test data...");
    
    // Create simple test data
    let event = TestSerializableStreamEvent {
        timestamp: 1000,
        before_window_data: vec![
            TestSerializableAttributeValue::String("test".to_string()),
            TestSerializableAttributeValue::Int(42),
        ],
        output_data: None,
        event_type: 0,
    };
    
    let chunk = TestSerializableSessionChunk {
        events: vec![event],
        start_timestamp: 1000,
        end_timestamp: 2000,
        alive_timestamp: 2000,
    };
    
    let container = TestSerializableSessionContainer {
        current_session: chunk.clone(),
        previous_session: TestSerializableSessionChunk {
            events: vec![],
            start_timestamp: 0,
            end_timestamp: 0,
            alive_timestamp: 0,
        },
    };
    
    let mut session_map = HashMap::new();
    session_map.insert("session1".to_string(), container);
    
    let state = TestSerializableSessionState {
        session_map,
        expired_event_chunk: TestSerializableSessionChunk {
            events: vec![],
            start_timestamp: 0,
            end_timestamp: 0,
            alive_timestamp: 0,
        },
    };
    
    println!("About to serialize...");
    
    match bincode::serialize(&state) {
        Ok(data) => {
            println!("Serialization successful! {} bytes", data.len());
        },
        Err(e) => {
            println!("Serialization failed: {}", e);
        }
    }
    
    println!("Done!");
}