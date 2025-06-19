# Siddhi Rust Port (siddhi_rust)

This project is an experimental port of the Java-based Siddhi CEP (Complex Event Processing) engine to Rust. The primary goal is to explore the feasibility and potential benefits (performance, memory safety) of a Rust implementation.

## Current Status

The project is in the early stages of porting. Key modules have been structurally translated, with foundational logic for some components in place.

*   **`siddhi-query-api` Module**: Largely ported. This module defines the abstract syntax tree (AST) and structures for representing Siddhi applications, stream definitions, queries, expressions, and execution plans. Most data structures have been translated to Rust structs and enums.
*   **`siddhi-query-compiler` Module**: Provides a LALRPOP-based parser for SiddhiQL.
    *   The `update_variables` function (for substituting environment/system variables in SiddhiQL strings) has been ported.
    *   Parsing now uses the grammar in `query_compiler/grammar.lalrpop` to build the AST.
*   **`siddhi-core` Module**: Foundational elements for a Phase 1 feature set (simple stateless queries like filters and projections) are structurally in place. This includes:
    *   **Configuration (`config`)**: `SiddhiContext` and `SiddhiAppContext` defined (many internal fields are placeholders for complex Java objects like persistence stores, data sources, executor services).
    *   **Events (`event`)**: `Event`, `AttributeValue`, `ComplexEvent` trait, and `StreamEvent` are defined. Placeholders for state/meta events exist.
    *   **Stream Handling (`stream`)**: Basic structures for `StreamJunction` (event routing) and `InputHandler` are defined. `StreamCallback` trait for output.
    *   **Expression Executors (`executor`)**: `ExpressionExecutor` trait defined. Implementations for constants, variables (simplified), basic math operators (+,-,*,/,mod), basic conditions (AND,OR,NOT,Compare,IsNull), and common functions (Coalesce, IfThenElse, UUID, InstanceOf*) are present.
    *   **Expression Parser (`util/parser/expression_parser.rs`)**: Initial recursive structure to convert `query_api::Expression` objects into `core::ExpressionExecutor`s.
    *   **Stream Processors (`query/processor`)**: `Processor` trait and `CommonProcessorMeta` struct.  In addition to `FilterProcessor` and `SelectProcessor`, the Rust port includes `LengthWindowProcessor`, `TimeWindowProcessor`, `JoinProcessor`, and processors for event patterns and sequences.  `InsertIntoStreamProcessor` handles output routing.
    *   **Runtime Parsers (`util/parser/siddhi_app_parser.rs`, `util/parser/query_parser.rs`)**: Build `SiddhiAppRuntime`s from the AST.  The parser supports windows, joins, patterns, sequences and incremental aggregations.
    *   **Runtime (`siddhi_app_runtime.rs`)**: `SiddhiAppRuntime` executes queries built by the parser, including windows, joins, patterns, sequences and aggregations.  Runtimes use the scheduler for time-based operations and can register callbacks for output.
*   **`SiddhiManager`**: Basic functionality for creating, retrieving, and shutting down `SiddhiAppRuntime` instances has been ported. Methods for managing extensions and data sources are placeholders pointing to `SiddhiContext`.
*   **Metrics and Fault Handling**: Simple in-memory metrics trackers are available and stream junctions can route faults to fault streams or an error store.

## Key Omissions, Simplifications, and Major TODOs

This port is **far from feature-complete** with the Java version. Users should be aware of the following critical missing pieces and simplifications:

*   **SiddhiQL String Parsing**: A LALRPOP-based parser converts SiddhiQL strings into the `query_api` AST.  The grammar covers streams, tables, windows, triggers, aggregations, queries and partitions (with optional `define` syntax) and supports aggregation store queries with `within`/`per` clauses, but still omits many advanced constructs.
*   **`ExpressionParser` Completeness**:
    *   **Variable Resolution**: Current logic is highly simplified for a single input stream. It does not handle joins, patterns, states, tables, window functions, or aggregation variable lookups. Attribute position and type resolution from complex contexts is a major TODO.
    *   **Function Handling**: Only a few common built-in functions are recognized. A full function lookup mechanism (including UDFs from `SiddhiContext`, script functions) and robust argument parsing/type checking is needed.
    *   **Type Checking & Coercion**: Rigorous Siddhi-specific type checking and coercion for all operators and functions is not yet implemented.
    *   **Error Handling**: Error reporting from parsing is basic (String-based).
*   **`ExpressionExecutor` Implementations**:
    *   `VariableExpressionExecutor`: `execute` method uses a simplified data access model (assumes data in `StreamEvent::output_data` or `before_window_data` via a simple index). Needs to correctly handle different event types, data arrays (input, output, before/after window data), and dynamic resolution (tables, stores).
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
    *   **Persistence**: Includes a `SnapshotService` and an in-memory `PersistenceStore`. Durable persistence stores are still pending.
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

### Dynamic Extension Loading

Extensions can be compiled into separate crates and loaded at runtime.  A library
must expose a `register_extension` function that registers factories with a
`SiddhiManager`.  The integration tests contain a sample dynamic extension under
`tests/custom_dyn_ext`.

```rust
let manager = SiddhiManager::new();
let lib_path = custom_dyn_ext::library_path();
manager
    .set_extension("custom", lib_path.to_str().unwrap().to_string())
    .unwrap();
```

Once loaded, the factories provided by the library can be used like any other
registered extension.

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

1.  **Stabilize Phase 1**: Make the `test_simple_filter_projection_query` compile and run successfully by fully implementing the simplified logic paths in `ExpressionParser`, `VariableExpressionExecutor`, `FilterProcessor`, `SelectProcessor`, and event data handling.
2.  **Basic Stateful Operations**: Introduce `LengthWindowProcessor` and other simple stateful processors.
3.  **Expand Core Logic**: Gradually implement more expression executors, stream processors, join capabilities, and aggregation functions.
4.  **SiddhiQL Parsing**: Continue expanding the Rust-based grammar to support more of the language and improve error reporting.

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
