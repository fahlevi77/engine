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