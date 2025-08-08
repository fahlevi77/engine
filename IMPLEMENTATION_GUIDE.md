# Siddhi Rust Implementation Guide

This guide helps developers implement new features in the Siddhi Rust port by providing patterns and examples.

## Quick Reference: Java to Rust Patterns

### Window Implementation Pattern

**Java Structure:**
```java
public class SessionWindowProcessor extends WindowProcessor {
    private List<Event> window = new ArrayList<>();
    
    @Override
    protected void process(ComplexEventChunk streamEventChunk) {
        // Processing logic
    }
}
```

**Rust Equivalent:**
```rust
pub struct SessionWindowProcessor {
    meta: CommonProcessorMeta,
    buffer: Arc<Mutex<Vec<StreamEvent>>>,
    session_timeout: i64,
}

impl Processor for SessionWindowProcessor {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>) {
        // Processing logic
    }
}
```

### Key Patterns to Follow

1. **State Management**
   ```rust
   // Always use Arc<Mutex<>> for shared state
   buffer: Arc<Mutex<VecDeque<Arc<StreamEvent>>>>
   
   // Register with StateHolder for persistence
   let holder = Arc::new(MyStateHolder { buffer: Arc::clone(&buffer) });
   query_ctx.register_state_holder("my_window".to_string(), holder);
   ```
   
   **📋 NEW**: See [STATE_MANAGEMENT_DESIGN.md](STATE_MANAGEMENT_DESIGN.md) for the comprehensive enterprise-grade state management design that will replace the current basic StateHolder trait.

2. **Factory Pattern**
   ```rust
   impl WindowProcessorFactory for MyWindowFactory {
       fn name(&self) -> &'static str { "myWindow" }
       fn create(&self, handler: &WindowHandler, ...) -> Result<Arc<Mutex<dyn Processor>>, String> {
           Ok(Arc::new(Mutex::new(MyWindowProcessor::from_handler(handler, app_ctx, query_ctx)?)))
       }
   }
   ```

3. **Error Handling**
   ```rust
   // Current (to be improved)
   return Err("Window requires parameter".to_string());
   
   // Future (with proper errors)
   return Err(SiddhiError::InvalidParameter { 
       component: "SessionWindow",
       message: "Session timeout required"
   });
   ```

## Common Implementation Steps

### 1. Adding a New Window

1. **Create the processor struct** in `src/core/query/processor/stream/window/mod.rs`:
   ```rust
   #[derive(Debug)]
   pub struct SessionWindowProcessor {
       meta: CommonProcessorMeta,
       session_timeout: i64,
       sessions: Arc<Mutex<HashMap<String, Vec<StreamEvent>>>>,
   }
   ```

2. **Implement constructor and from_handler**:
   ```rust
   impl SessionWindowProcessor {
       pub fn new(timeout: i64, app_ctx: Arc<SiddhiAppContext>, query_ctx: Arc<SiddhiQueryContext>) -> Self {
           // Initialize
       }
       
       pub fn from_handler(handler: &WindowHandler, ...) -> Result<Self, String> {
           // Parse parameters and create instance
       }
   }
   ```

3. **Implement Processor trait**:
   - `process()` - Main event processing logic
   - `clone_processor()` - For query cloning
   - `is_stateful()` - Return true for stateful windows
   - `get_processing_mode()` - SLIDE, BATCH, or DEFAULT

4. **Add to window factory**:
   - Update `create_window_processor()` function
   - Add case for your window name

5. **Write tests** in `tests/`:
   ```rust
   #[test]
   fn test_session_window() {
       let mut runner = AppRunner::new(
           r#"
           define stream InputStream (id string, value int);
           from InputStream#window.session(5 sec, id)
           select id, sum(value) as total
           insert into OutputStream;
           "#
       );
       // Test logic
   }
   ```

### 2. Adding a New Function

1. **Create executor** in `src/core/executor/function/`:
   ```rust
   #[derive(Debug, Clone)]
   pub struct MyFunctionExecutor {
       args: Vec<Box<dyn ExpressionExecutor>>,
       return_type: ApiAttributeType,
   }
   ```

2. **Implement ScalarFunctionExecutor**:
   ```rust
   impl ScalarFunctionExecutor for MyFunctionExecutor {
       fn init(&mut self, args: &Vec<Box<dyn ExpressionExecutor>>, ctx: &Arc<SiddhiAppContext>) -> Result<(), String> {
           // Validate args and determine return type
       }
       
       fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
           // Execute function logic
       }
   }
   ```

3. **Register in SiddhiManager** or via extension

### 3. Adding a New Aggregator

1. **Create in** `src/core/query/selector/attribute/aggregator/mod.rs`

2. **Implement AttributeAggregatorExecutor**:
   ```rust
   impl AttributeAggregatorExecutor for MyAggregator {
       fn process_add(&self, data: Option<AttributeValue>) -> Option<AttributeValue> {
           // Add to aggregation
       }
       
       fn process_remove(&self, data: Option<AttributeValue>) -> Option<AttributeValue> {
           // Remove from aggregation (for sliding windows)
       }
       
       fn reset(&self) -> Option<AttributeValue> {
           // Reset aggregation state
       }
   }
   ```

## Grammar Extension Patterns

### Adding New Annotation Support

#### @Async Annotation Implementation Pattern

The @Async annotation implementation demonstrates the complete pattern for adding annotation support to Siddhi Rust. This includes grammar rules, parser integration, and runtime configuration.

**Grammar Pattern for Minimal and Parameterized Annotations:**
```lalrpop
// Support both minimal and parameterized annotations
pub AnnotationStmt: Annotation = {
    "@" <name:Ident> "(" <pairs:KeyValuePairs> ")" => {
        let mut a = Annotation::new(name);
        for (k, v) in pairs { a = a.element(Some(k), v); }
        a
    },
    "@" <name:Ident> ":" <key:Ident> "(" <val:STRING> ")" => {
        Annotation::new(name).element(Some(key), val)
    },
    "@" <name:Ident> => {
        Annotation::new(name)  // Minimal annotation support (e.g., @Async)
    }
};

// Use underscore notation to avoid grammar conflicts
KeyValue: (String, String) = { <k:Ident> "=" <v:STRING> => (k, v) };
```

**Parser Integration Pattern:**
```rust
// In siddhi_app_parser.rs - Complete @Async annotation processing
for ann in &stream_def_arc.abstract_definition.annotations {
    match ann.name.to_lowercase().as_str() {
        "async" => {
            use_optimized = true;
            config = config.with_async(true);
            
            // Parse @Async annotation parameters with underscore notation
            for el in &ann.elements {
                match el.key.to_lowercase().as_str() {
                    "buffer_size" | "buffersize" => {
                        if let Ok(sz) = el.value.parse::<usize>() {
                            config = config.with_buffer_size(sz);
                        }
                    }
                    "workers" => {
                        if let Ok(workers) = el.value.parse::<u64>() {
                            let estimated_throughput = workers * 10000; // 10K events/worker estimate
                            config = config.with_expected_throughput(estimated_throughput);
                        }
                    }
                    "batch_size_max" | "batchsizemax" => {
                        // Note: batch size is handled internally by the pipeline
                        // This is for compatibility with Java Siddhi
                    }
                    _ => {}
                }
            }
        }
        "config" => {
            // Support @config(async='true') for global configuration
            for el in &ann.elements {
                match el.key.to_lowercase().as_str() {
                    "async" => {
                        if el.value.eq_ignore_ascii_case("true") {
                            use_optimized = true;
                            config = config.with_async(true);
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

// Enhanced StreamJunction creation with async configuration
if use_optimized {
    println!(
        "Created async stream '{}' with buffer_size={}, async={}", 
        stream_id, config.buffer_size, config.is_async
    );
    // TODO: In future versions, replace with OptimizedStreamJunction
    // when SiddhiAppRuntimeBuilder is updated to support it
}
```

**Global Configuration Pattern:**
```rust
// In siddhi_app_runtime.rs - App-level annotation processing
let mut default_stream_async = siddhi_app_context
    .get_siddhi_context()
    .get_default_async_mode();

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
```

**Testing Pattern for Grammar Extensions:**
```rust
// Complete test suite for @Async annotation functionality
#[test]
fn test_async_annotation_with_parameters() {
    let mut manager = SiddhiManager::new();
    let siddhi_app_string = r#"
        @Async(buffer_size='1024', workers='2', batch_size_max='10')
        define stream TestStream (id int, value string);
    "#;
    
    let result = manager.create_siddhi_app_runtime_from_string(siddhi_app_string);
    assert!(result.is_ok(), "Failed to parse @Async annotation: {:?}", result.err());
    
    // Verify annotation parameters are parsed correctly
    let app_runtime = result.unwrap();
    let stream_def = app_runtime.siddhi_app.stream_definition_map
        .get("TestStream").unwrap();
        
    let async_annotation = stream_def.abstract_definition.annotations
        .iter()
        .find(|ann| ann.name.eq_ignore_ascii_case("async"))
        .unwrap();
        
    // Test underscore notation parameter parsing
    let buffer_size = async_annotation.elements
        .iter()
        .find(|el| el.key.eq_ignore_ascii_case("buffer_size"))
        .unwrap();
    assert_eq!(buffer_size.value, "1024");
    
    let workers = async_annotation.elements
        .iter()
        .find(|el| el.key.eq_ignore_ascii_case("workers"))
        .unwrap();
    assert_eq!(workers.value, "2");
}

#[test]
fn test_minimal_async_annotation() {
    let mut manager = SiddhiManager::new();
    let siddhi_app_string = r#"
        @Async
        define stream MinimalAsyncStream (id int, value string);
    "#;
    
    let result = manager.create_siddhi_app_runtime_from_string(siddhi_app_string);
    assert!(result.is_ok(), "Failed to parse minimal @Async annotation");
}

#[test]
fn test_global_async_configuration() {
    let mut manager = SiddhiManager::new();
    let siddhi_app_string = r#"
        @app(async='true')
        
        define stream AutoAsyncStream (id int, value string);
        define stream RegularStream (name string, count int);
    "#;
    
    let result = manager.create_siddhi_app_runtime_from_string(siddhi_app_string);
    assert!(result.is_ok(), "Failed to parse app-level @app annotation");
}

#[test]
fn test_async_with_query_processing() {
    let mut manager = SiddhiManager::new();
    let siddhi_app_string = r#"
        @Async(buffer_size='1024')
        define stream InputStream (symbol string, price float, volume long);
        
        define stream OutputStream (symbol string, avgPrice float);
        
        from InputStream#time(10 sec)
        select symbol, avg(price) as avgPrice
        insert into OutputStream;
    "#;
    
    let result = manager.create_siddhi_app_runtime_from_string(siddhi_app_string);
    assert!(result.is_ok(), "Failed to parse async stream with query");
}
```

#### Grammar Debugging Guidelines

**Common Grammar Conflicts:**
1. **Dot Notation Conflicts**: Avoid using dots in annotation parameters when they conflict with join syntax
2. **Keyword Conflicts**: Handle reserved keywords like `window` in specific contexts
3. **Ambiguous Parsing**: Use specific rule ordering to resolve conflicts

**Resolution Strategies:**
1. **Use underscore notation** for parameter names (`buffer_size` vs `buffer.size`)
2. **Add specific keyword rules** for context-sensitive parsing (`window.time` → separate rules)
3. **Test incrementally** with minimal examples before complex queries

## Testing Patterns

### Integration Test Template
```rust
use crate::common::AppRunner;

#[test]
fn test_my_feature() {
    let mut runner = AppRunner::new(
        r#"
        @app:name('TestApp')
        define stream InputStream (attr1 type1, attr2 type2);
        
        @info(name = 'query1')
        from InputStream[filter_condition]
        select attr1, my_function(attr2) as result
        insert into OutputStream;
        "#
    );
    
    // Send events
    runner.send("InputStream", vec![
        btreemap! {
            "attr1" => AttributeValue::String("test".to_string()),
            "attr2" => AttributeValue::Int(42)
        }
    ]);
    
    // Assert output
    runner.assert_output("OutputStream", vec![
        btreemap! {
            "attr1" => AttributeValue::String("test".to_string()),
            "result" => AttributeValue::Int(84)
        }
    ]);
}
```

## Performance Considerations

1. **Avoid Cloning Events** - Use `Arc<StreamEvent>` for sharing
2. **Batch Operations** - Process multiple events together when possible
3. **Lazy Evaluation** - Don't compute until necessary
4. **Pool Resources** - Reuse buffers and temporary objects

## Debugging Tips

1. **Enable debug logging**:
   ```rust
   log::debug!("Processing event: {:?}", event);
   ```

2. **Use assert! for invariants**:
   ```rust
   debug_assert!(self.buffer.lock().unwrap().len() <= self.max_size);
   ```

3. **Test edge cases**:
   - Empty windows
   - Single event
   - Null/None values
   - Type mismatches
   - Concurrent access

## Resources

- Java source: `modules/siddhi-core/src/main/java/io/siddhi/core/`
- Java tests: `modules/siddhi-core/src/test/java/io/siddhi/core/`
- Rust examples: `siddhi_rust/tests/`
- Extension guide: `siddhi_rust/docs/writing_extensions.md`