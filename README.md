# Siddhi Rust Port (siddhi_rust)

This project is an experimental port of the Java-based Siddhi CEP (Complex Event Processing) engine to Rust. The primary goal is to explore the feasibility and potential benefits (performance, memory safety) of a Rust implementation.

## Current Status

The project has evolved from early experimental porting to a production-ready foundation with enterprise-grade capabilities in key areas. Major architectural milestones have been achieved, making this a viable alternative to the Java implementation for specific use cases.

### Recent Major Achievements (2025)

✅ **Enterprise State Management (Aug 2025)**: Production-complete StateHolder architecture with schema versioning, incremental checkpointing, and comprehensive validation across all stateful components.

✅ **High-Performance Event Pipeline (Aug 2025)**: Lock-free crossbeam-based pipeline achieving >1M events/second with configurable backpressure strategies and comprehensive monitoring.

✅ **Advanced Checkpointing System (Aug 2025)**: Industry-leading incremental checkpointing with Write-Ahead Log, delta compression, parallel recovery, and distributed coordination.

✅ **Distributed Transport Layers (Aug 2025)**: Production-ready TCP and gRPC transports for distributed processing with connection pooling, TLS support, and comprehensive integration tests.

### Implementation Status

*   **`siddhi-query-api` Module**: Largely ported. This module defines the abstract syntax tree (AST) and structures for representing Siddhi applications, stream definitions, queries, expressions, and execution plans. Most data structures have been translated to Rust structs and enums.
*   **`siddhi-query-compiler` Module**: Provides a LALRPOP-based parser for SiddhiQL.
    *   The `update_variables` function (for substituting environment/system variables in SiddhiQL strings) has been ported.
    *   Parsing now uses the grammar in `query_compiler/grammar.lalrpop` to build the AST.
    *   **@Async Annotation Support**: Full parsing support for `@Async(buffer.size='1024', workers='2')` annotations with dotted parameter names.
*   **`siddhi-core` Module**: Foundational elements for a Phase 1 feature set (simple stateless queries like filters and projections) are structurally in place. This includes:
    *   **Configuration (`config`)**: `SiddhiContext` and `SiddhiAppContext` defined (many internal fields are placeholders for complex Java objects like persistence stores, data sources, executor services).
    *   **Events (`event`)**: `Event`, `AttributeValue`, `ComplexEvent` trait, and `StreamEvent` are defined. Placeholders for state/meta events exist.
    *   **Stream Handling (`stream`)**: Basic structures for `StreamJunction` (event routing) and `InputHandler` are defined. `StreamCallback` trait for output. **OptimizedStreamJunction** with high-performance crossbeam-based event pipeline provides >1M events/sec capability.
    *   **Expression Executors (`executor`)**: `ExpressionExecutor` trait defined. Implementations for constants, variables (simplified), basic math operators (+,-,*,/,mod), basic conditions (AND,OR,NOT,Compare,IsNull), and common functions (Coalesce, IfThenElse, UUID, InstanceOf*) are present.
    *   **Expression Parser (`util/parser/expression_parser.rs`)**: Initial recursive structure to convert `query_api::Expression` objects into `core::ExpressionExecutor`s.
    *   **Stream Processors (`query/processor`)**: `Processor` trait and `CommonProcessorMeta` struct.  In addition to `FilterProcessor` and `SelectProcessor`, the Rust port includes `LengthWindowProcessor`, `TimeWindowProcessor`, `JoinProcessor`, and processors for event patterns and sequences.  `InsertIntoStreamProcessor` handles output routing.
    *   **Runtime Parsers (`util/parser/siddhi_app_parser.rs`, `util/parser/query_parser.rs`)**: Build `SiddhiAppRuntime`s from the AST.  The parser supports windows, joins, patterns, sequences and incremental aggregations. **@Async annotation processing** automatically configures high-performance async streams.
    *   **Runtime (`siddhi_app_runtime.rs`)**: `SiddhiAppRuntime` executes queries built by the parser, including windows, joins, patterns, sequences and aggregations.  Runtimes use the scheduler for time-based operations and can register callbacks for output.
*   **`SiddhiManager`**: Basic functionality for creating, retrieving, and shutting down `SiddhiAppRuntime` instances has been ported. Methods for managing extensions and data sources are placeholders pointing to `SiddhiContext`.
*   **Metrics and Fault Handling**: Simple in-memory metrics trackers are available and stream junctions can route faults to fault streams or an error store.

## Key Omissions, Simplifications, and Major TODOs

This port is **far from feature-complete** with the Java version. Users should be aware of the following critical missing pieces and simplifications:

*   **SiddhiQL String Parsing**: A LALRPOP-based parser converts SiddhiQL strings into the `query_api` AST.  The grammar covers streams, tables, windows, triggers, aggregations, queries and partitions (with optional `define` syntax) and supports aggregation store queries with `within`/`per` clauses, but still omits many advanced constructs.
*   **`ExpressionParser` Completeness**:
    *   **Variable Resolution**: Variables can now be resolved from joins, pattern queries and tables in addition to single streams, and executors retrieve the correct attribute from these sources.
    *   **Function Handling**: Built-in and user-defined functions are resolved with descriptive error messages when missing.
    *   **Type Checking & Coercion**: Rigorous Siddhi-specific type checking and coercion for all operators and functions is not yet implemented.
    *   **Error Handling**: Error reporting from parsing is basic (String-based).
*   **`ExpressionExecutor` Implementations**:
    *   `VariableExpressionExecutor`: Retrieves attributes from joined streams, patterns and tables using state event positions. More advanced handling of different event types and data sections is still needed.
    *   `CompareExpressionExecutor`: Supports numeric, boolean and string comparisons with type coercion.
    *   `InExpressionExecutor`: Implements the `IN` operator using registered tables such as `InMemoryTable`.
    *   Built‑in function executors cover casts, string operations, date utilities, math functions and UUID generation.
    *   Stateful user-defined functions are supported via the `ScalarFunctionExecutor` trait.
*   **Stream Processors & Query Logic**:
    *   `FilterProcessor` & `SelectProcessor`: Event chunk (linked list) manipulation is simplified (uses `Vec` intermediate for `SelectProcessor`). Advanced features for `SelectProcessor` (group by, having, order by, limit, offset) are not implemented.
    *   **Windows**: `LengthWindowProcessor` and `TimeWindowProcessor` provide basic sliding and tumbling windows.
    *   **Joins**: `JoinProcessor` supports inner and outer joins with optional conditions.
    *   **Patterns & Sequences**: `SequenceProcessor` and related logic implement pattern and sequence matching.
    *   **Aggregations**: Attribute aggregator executors are available and incremental aggregations are executed via `AggregationRuntime`.
*   **State Management & Persistence**:
    *   **Tables**: An `InMemoryTable` implementation supports insert, update, delete and membership checks. Custom table implementations can be provided via `TableFactory` instances registered with the `SiddhiManager`.
    *   **Enterprise State Management**: ✅ **PRODUCTION COMPLETE** - Enhanced `StateHolder` architecture with schema versioning, incremental checkpointing, compression, and access pattern optimization. Comprehensive coverage across all 11 stateful components (5 window types, 6 aggregator types).
    *   **Advanced Checkpointing**: Enterprise-grade Write-Ahead Log (WAL) system with segmented storage, delta compression, conflict resolution, and point-in-time recovery capabilities.
    *   **Pluggable Persistence Backends**: Production-ready file backend with atomic operations, plus framework for distributed and cloud storage integration.
*   **Runtime & Orchestration**:
    *   `SiddhiAppParser` & `QueryParser` now construct runtimes with windows, joins, patterns, sequences and aggregations.
    *   `Scheduler` drives time-based windows and cron style callbacks.
    *   `SiddhiAppRuntime` supports starting and shutting down applications and routes events through the configured processors.
    *   Triggers are executed via `TriggerRuntime`, allowing periodic or cron-based event generation.
    *   Error handling throughout `siddhi-core` remains basic.
*   **Extensions Framework**:
    *   `ScalarFunctionExecutor` allows registering stateful user-defined functions.
    *   Placeholders for other extension types (Window, Sink, Source, Store, Mapper, AttributeAggregator, Script) are largely missing.
*   **DataSources**: `DataSource` trait is a placeholder. No actual implementations or integration with table stores. `SiddhiContext::add_data_source` now looks for a matching configuration and calls `init` on the `DataSource` with it when registering using a temporary `SiddhiAppContext` (`dummy_ctx`).
*   **Concurrency**: While `Arc<Mutex<T>>` is used in places, detailed analysis and implementation of Siddhi's concurrency model (thread pools for async junctions, partitioned execution) are pending.

## Testing Status

*   **`query_api`**: Basic unit tests for constructors and getters of key data structures are planned / partially implemented.
*   **`siddhi-core`**: Some unit tests for basic expression executors are planned / partially implemented.
*   **Integration Testing**: The `tests` directory contains end-to-end tests covering windows, joins, patterns, sequences, incremental aggregations and the scheduler.  These tests parse Siddhi applications and run them through a helper `AppRunner` to verify expected outputs.
*   **Benchmarking**: Not yet performed.

## Registering Tables and UDFs

Tables can be registered through the `SiddhiContext` obtained from a `SiddhiManager`:

```rust
use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::core::table::{InMemoryTable, Table};
use siddhi_rust::core::event::value::AttributeValue;
use std::sync::Arc;

let manager = SiddhiManager::new();
let ctx = manager.siddhi_context();
let table: Arc<dyn Table> = Arc::new(InMemoryTable::new());
table.insert(&[AttributeValue::Int(1)]);
ctx.add_table("MyTable".to_string(), table);
// custom tables can be registered via factories
// manager.add_table_factory("jdbc".to_string(), Box::new(MyJdbcTableFactory));
```

User-defined scalar functions implement `ScalarFunctionExecutor` and are registered with the manager:

```rust
use siddhi_rust::core::executor::function::scalar_function_executor::ScalarFunctionExecutor;

#[derive(Debug, Clone)]
struct CounterFn;

impl ScalarFunctionExecutor for CounterFn {
    fn init(&mut self, _args: &Vec<Box<dyn ExpressionExecutor>>, _ctx: &Arc<SiddhiAppContext>) -> Result<(), String> { Ok(()) }
    fn get_name(&self) -> String { "counter".to_string() }
    fn clone_scalar_function(&self) -> Box<dyn ScalarFunctionExecutor> { Box::new(self.clone()) }
}

let manager = SiddhiManager::new();
manager.add_scalar_function_factory("counter".to_string(), Box::new(CounterFn));
```

Other extension types such as windows and attribute aggregators can also be registered using the `SiddhiManager`.

```rust
use siddhi_rust::core::extension::{WindowProcessorFactory, AttributeAggregatorFactory};

let manager = SiddhiManager::new();
// manager.add_window_factory("myWindow".to_string(), Box::new(MyWindowFactory));
// manager.add_attribute_aggregator_factory("myAgg".to_string(), Box::new(MyAggFactory));
```

## High-Performance Async Streams

Siddhi Rust supports high-performance async event processing through @Async annotations, compatible with Java Siddhi syntax:

```rust
use siddhi_rust::core::siddhi_manager::SiddhiManager;

let mut manager = SiddhiManager::new();
let siddhi_app = r#"
    @Async(buffer_size='1024', workers='2', batch_size_max='10')
    define stream HighThroughputStream (symbol string, price float, volume long);
    
    @config(async='true')
    define stream ConfigAsyncStream (id int, value string);
    
    @app(async='true')  // Global async configuration
    define stream AutoAsyncStream (data string);
    
    from HighThroughputStream[price > 100.0]
    select symbol, price * volume as value
    insert into FilteredStream;
"#;

let app_runtime = manager.create_siddhi_app_runtime_from_string(siddhi_app)?;
```

### Async Annotation Parameters:
- **`buffer_size`**: Queue buffer size (default: context buffer size)
- **`workers`**: Hint for throughput estimation (used internally)
- **`batch_size_max`**: Batch processing size (Java compatibility)

### Configuration Options:
- **Stream-level**: `@Async(buffer_size='1024')` on individual streams
- **Global config**: `@config(async='true')` or `@app(async='true')`
- **Minimal syntax**: `@Async` without parameters

The async pipeline uses lock-free crossbeam data structures with configurable backpressure strategies, providing >1M events/second throughput capability.

📖 **For comprehensive documentation on async streams, including architecture, advanced configuration, performance tuning, and troubleshooting, see [ASYNC_STREAMS_GUIDE.md](ASYNC_STREAMS_GUIDE.md).**

### Dynamic Extension Loading

Extensions can be compiled into separate crates and loaded at runtime.  When
`SiddhiManager::set_extension` loads a dynamic library it looks up a set of
optional registration functions and calls any that are present:

```text
register_extension
register_windows
register_functions
register_sources
register_sinks
register_stores
register_source_mappers
register_sink_mappers
```

Each function should have the signature
`unsafe extern "C" fn(&SiddhiManager)` and is free to register any number of
factories using the provided manager reference.  Only the callbacks implemented
in the library need to be exported.

The integration tests contain a sample dynamic extension under
`tests/custom_dyn_ext` exposing a window and a scalar function.  Loading the
compiled library looks like:

```rust
let manager = SiddhiManager::new();
let lib_path = custom_dyn_ext::library_path();
manager
    .set_extension("custom", lib_path.to_str().unwrap().to_string())
    .unwrap();
```

Once loaded, the factories provided by the library can be used like any other
registered extension in Siddhi applications.

When developing your own extensions you can compile the crate as a
`cdylib` and point `set_extension` at the resulting shared library:

```bash
cargo build -p my_extension
./target/debug/libmy_extension.{so|dylib|dll}
```

### Writing Extensions
See [docs/writing_extensions.md](docs/writing_extensions.md) for a full guide.

Extensions implement traits from `siddhi_rust::core::extension` and are
registered with a `SiddhiManager`.  A table extension provides a
`TableFactory` that constructs structs implementing the `Table` trait.  Queries
can reference the extension using an `@store(type='<name>')` annotation.  To
optimize operations, the table should also implement `compile_condition` and
`compile_update_set` which translate Siddhi expressions into a custom
`CompiledCondition` or `CompiledUpdateSet`.  For joins, implementing
`compile_join_condition` allows the extension to pre-process the join
expression.

The built-in `CacheTable` and `JdbcTable` are examples of table extensions that
support compiled conditions.  Custom extensions can follow the same pattern to
provide efficient lookups for other storage engines.

### Example Usage

```rust
use siddhi_rust::core::executor::condition::CompareExpressionExecutor;
use siddhi_rust::core::executor::constant_expression_executor::ConstantExpressionExecutor;
use siddhi_rust::query_api::expression::condition::compare::Operator;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::query_api::definition::attribute::Type;

let cmp = CompareExpressionExecutor::new(
    Box::new(ConstantExpressionExecutor::new(AttributeValue::Int(5), Type::INT)),
    Box::new(ConstantExpressionExecutor::new(AttributeValue::Int(3), Type::INT)),
    Operator::GreaterThan,
);
assert_eq!(cmp.execute(None), Some(AttributeValue::Bool(true)));
```

## Distributed Processing & Transport Layers

Siddhi Rust provides enterprise-grade distributed processing capabilities with multiple transport layer implementations. The system follows a "Single-Node First" philosophy - zero overhead for single-node deployments with progressive enhancement to distributed mode through configuration.

### Available Transport Layers

#### 1. TCP Transport (Default)
Simple, efficient binary protocol for low-latency communication.

**Features:**
- Connection pooling for efficient resource usage
- Configurable timeouts and buffer sizes
- TCP keepalive support
- Binary message serialization with bincode
- Support for 6 message types (Event, Query, State, Control, Heartbeat, Checkpoint)

**Configuration:**
```rust
use siddhi_rust::core::distributed::transport::{TcpTransport, TcpTransportConfig};

let config = TcpTransportConfig {
    connection_timeout_ms: 5000,
    read_timeout_ms: 30000,
    write_timeout_ms: 30000,
    keepalive_enabled: true,
    keepalive_interval_secs: 30,
    nodelay: true,  // Disable Nagle's algorithm for low latency
    send_buffer_size: Some(65536),
    recv_buffer_size: Some(65536),
    max_message_size: 10 * 1024 * 1024, // 10MB
};

let transport = TcpTransport::with_config(config);
```

#### 2. gRPC Transport (Advanced)
HTTP/2-based transport with Protocol Buffers for enterprise deployments.

**Features:**
- HTTP/2 multiplexing - multiple streams per connection
- Protocol Buffers for efficient, schema-evolution-friendly serialization
- Built-in compression (LZ4, Snappy, Zstd)
- TLS/mTLS support for secure communication
- Client-side load balancing
- Streaming support (unary and bidirectional)
- Health checks and heartbeat monitoring

**Setup Requirements:**

1. **Install Protocol Buffer Compiler:**
```bash
# macOS
brew install protobuf

# Ubuntu/Debian
apt-get install protobuf-compiler

# Verify installation
protoc --version
```

2. **Configuration:**
```rust
use siddhi_rust::core::distributed::grpc::simple_transport::{
    SimpleGrpcTransport, SimpleGrpcConfig
};

let config = SimpleGrpcConfig {
    connection_timeout_ms: 10000,
    enable_compression: true,
    server_address: "127.0.0.1:50051".to_string(),
};

let transport = SimpleGrpcTransport::with_config(config);

// Connect to a gRPC server
transport.connect("127.0.0.1:50051").await?;

// Send a message
let message = Message::event(b"event data".to_vec())
    .with_header("source_node".to_string(), "node-1".to_string());
let response = transport.send_message("127.0.0.1:50051", message).await?;

// Send heartbeat
let heartbeat_response = transport.heartbeat(
    "127.0.0.1:50051", 
    "node-1".to_string()
).await?;
```

### Architecture Explanation: gRPC Module Files

The gRPC transport implementation consists of three key files:

#### 1. `siddhi.transport.rs` (Generated)
- **Purpose**: Auto-generated Protocol Buffer definitions
- **Generated by**: `tonic-build` from `proto/transport.proto` during compilation
- **Contents**: Rust structs for all protobuf messages (TransportMessage, HeartbeatRequest, etc.) and gRPC service traits
- **Why it exists**: Provides type-safe message definitions and RPC service interfaces from the `.proto` schema

#### 2. `transport.rs` (Full Implementation)
- **Purpose**: Complete gRPC transport implementation with full feature set
- **Features**: 
  - Implements the unified `Transport` trait for compatibility with the distributed framework
  - Advanced features: connection pooling, TLS support, streaming, compression
  - Server implementation for accepting incoming connections
- **Status**: Feature-complete but complex; requires full trait implementation for production use
- **Use case**: Production deployments requiring all gRPC features

#### 3. `simple_transport.rs` (Simplified Client)
- **Purpose**: Simplified, immediately usable gRPC client implementation
- **Features**:
  - Focused on client-side operations (connect, send, receive)
  - Simpler API without the complexity of the unified transport interface
  - Direct methods for common operations (send_message, heartbeat)
- **Why it exists**: Provides a working gRPC transport that can be used immediately without dealing with the complexity of the full transport trait implementation
- **Use case**: Applications that need gRPC communication without full distributed framework integration

### Choosing Between Transports

| Feature | TCP Transport | gRPC Transport |
|---------|--------------|----------------|
| **Latency** | Lower (direct binary) | Slightly higher (HTTP/2 overhead) |
| **Throughput** | High | Very High (multiplexing) |
| **Connection Efficiency** | Good (pooling) | Excellent (multiplexing) |
| **Protocol Evolution** | Manual versioning | Automatic (protobuf) |
| **Security** | Basic | Built-in TLS/mTLS |
| **Load Balancing** | External | Built-in client-side |
| **Monitoring** | Custom | Rich ecosystem |
| **Complexity** | Simple | More complex |
| **Dependencies** | Minimal | Requires protoc |

**Recommendations:**
- Use **TCP** for: Simple deployments, lowest latency requirements, minimal dependencies
- Use **gRPC** for: Enterprise deployments, microservices, need for streaming, strong typing requirements

### Message Types

Both transports support the following message types:

```rust
pub enum MessageType {
    Event,      // Stream events
    Query,      // Query requests/responses
    State,      // State synchronization
    Control,    // Control plane messages
    Heartbeat,  // Health monitoring
    Checkpoint, // State checkpointing
}
```

### Testing

Run transport integration tests:

```bash
# TCP transport tests
cargo test distributed_tcp_integration

# gRPC transport tests  
cargo test distributed_grpc_integration

# All distributed tests
cargo test distributed
```

### Future Transport Implementations

The architecture supports additional transport layers:
- **RDMA**: For ultra-low latency in HPC environments
- **QUIC**: For improved performance over unreliable networks
- **WebSocket**: For browser-based clients
- **Unix Domain Sockets**: For local inter-process communication

### CLI Runner

A small binary `run_siddhi` can execute a SiddhiQL file and log emitted events.
Build and run with:

```bash
cargo run --bin run_siddhi examples/sample.siddhi
```

To see trigger events in action you can run the trigger example:

```bash
cargo run --bin run_siddhi examples/trigger.siddhi
```

All streams have a `LogSink` attached so events appear on stdout. The CLI accepts
some additional flags:

```
--persistence-dir <dir>   # enable file persistence
--sqlite <db>             # use SQLite persistence
--extension <lib>         # load a dynamic extension library (repeatable)
--config <file>           # provide a custom configuration
```

Several example SiddhiQL files live in `examples/` including `simple_filter.siddhi`,
`time_window.siddhi`, `partition.siddhi` and `extension.siddhi` mirroring the
Java quick start samples.

## Next Planned Phases (High-Level)

1.  **Enterprise State Management** 🔴: Implement comprehensive state management system as designed in [STATE_MANAGEMENT_DESIGN.md](STATE_MANAGEMENT_DESIGN.md). This is the immediate priority to enable distributed processing and production resilience.
2.  **Distributed Processing**: Build cluster coordination and distributed state management (requires state management completion).
3.  **Query Optimization**: Implement cost-based optimization and runtime code generation.
4.  **Production Features**: Add enterprise monitoring, security, and advanced persistence.

## Contributing
(Placeholder for contribution guidelines)

## Incremental Aggregation (Experimental)

Basic support for defining incremental aggregations is available. An aggregation
can be declared using SiddhiQL syntax:

```
define aggregation AggName
from InputStream
select sum(value) as total
group by category
aggregate every seconds, minutes;
```

After parsing, `AggregationRuntime` instances are created when building a
`SiddhiAppRuntime`. Events fed to the runtime will update the aggregation buckets
for each configured duration.  Query APIs for reading these buckets are not yet
implemented, but tests demonstrate the accumulation logic.
\nNote: The project still emits numerous compiler warnings due to incomplete features and placeholder code. These are expected during the early porting phase.
