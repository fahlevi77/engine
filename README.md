# Siddhi Rust Port (siddhi_rust)

This project is an experimental port of the Java-based Siddhi CEP (Complex Event Processing) engine to Rust. The primary goal is to explore the feasibility and potential benefits (performance, memory safety) of a Rust implementation.

## Current Status

The project is in the early stages of porting. Key modules have been structurally translated, with foundational logic for some components in place.

*   **`siddhi-query-api` Module**: Largely ported. This module defines the abstract syntax tree (AST) and structures for representing Siddhi applications, stream definitions, queries, expressions, and execution plans. Most data structures have been translated to Rust structs and enums.
*   **`siddhi-query-compiler` Module**: This module is currently a placeholder.
    *   The `update_variables` function (for substituting environment/system variables in SiddhiQL strings) has been ported.
    *   **Actual SiddhiQL string parsing (ANTLR based in Java) is NOT yet implemented in Rust.** Placeholder functions exist in the Rust `query_compiler` that currently return "Not Implemented" errors.
*   **`siddhi-core` Module**: Foundational elements for a Phase 1 feature set (simple stateless queries like filters and projections) are structurally in place. This includes:
    *   **Configuration (`config`)**: `SiddhiContext` and `SiddhiAppContext` defined (many internal fields are placeholders for complex Java objects like persistence stores, data sources, executor services).
    *   **Events (`event`)**: `Event`, `AttributeValue`, `ComplexEvent` trait, and `StreamEvent` are defined. Placeholders for state/meta events exist.
    *   **Stream Handling (`stream`)**: Basic structures for `StreamJunction` (event routing) and `InputHandler` are defined. `StreamCallback` trait for output.
    *   **Expression Executors (`executor`)**: `ExpressionExecutor` trait defined. Implementations for constants, variables (simplified), basic math operators (+,-,*,/,mod), basic conditions (AND,OR,NOT,Compare,IsNull), and common functions (Coalesce, IfThenElse, UUID, InstanceOf*) are present.
    *   **Expression Parser (`util/parser/expression_parser.rs`)**: Initial recursive structure to convert `query_api::Expression` objects into `core::ExpressionExecutor`s.
    *   **Stream Processors (`query/processor`)**: `Processor` trait, `CommonProcessorMeta` struct. `FilterProcessor` (for WHERE clauses) and `SelectProcessor` (for SELECT clauses, simple projections only) are defined with basic logic. `InsertIntoStreamProcessor` for output.
    *   **Runtime Parsers (`util/parser/siddhi_app_parser.rs`, `util/parser/query_parser.rs`)**: Skeleton structure to take a `query_api::SiddhiApp` and build a runtime plan using the above core components (StreamJunctions, Processors).
    *   **Runtime (`siddhi_app_runtime.rs`)**: Basic `SiddhiAppRuntime` that can be constructed (via `SiddhiAppParser`), can receive input events, and add callbacks for output.
*   **`SiddhiManager`**: Basic functionality for creating, retrieving, and shutting down `SiddhiAppRuntime` instances has been ported. Methods for managing extensions and data sources are placeholders pointing to `SiddhiContext`.

## Key Omissions, Simplifications, and Major TODOs

This port is **far from feature-complete** with the Java version. Users should be aware of the following critical missing pieces and simplifications:

*   **No SiddhiQL String Parsing**: The `query_compiler` cannot currently parse SiddhiQL query strings into the `query_api::SiddhiApp` representation. This is the largest current omission for usability.
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
    *   **Windows**: No window processors (`TimeWindow`, `LengthWindow`, etc.) are ported. This is a major feature set.
    *   **Joins**: No join processors or join logic implemented.
    *   **Patterns & Sequences**: No pattern or sequence processors implemented.
    *   **Aggregations**: Basic attribute aggregator executors (sum, avg, count, distinctCount, min/max and forever variants) are implemented for use in queries.
*   **State Management & Persistence**:
    *   **Tables**: An `InMemoryTable` implementation supports insert, update, delete and membership checks.
    *   **Persistence**: `SnapshotService` and `PersistenceStore` framework is not implemented. No state persistence or recovery.
*   **Runtime & Orchestration**:
    *   `SiddhiAppParser` & `QueryParser`: Can only handle very simple queries (single stream, optional filter, simple select, insert into). Cannot parse partitions, windows, joins, patterns, sequences, tables, aggregations.
    *   `Scheduler`: Not implemented (needed for time-based windows and triggers).
    *   `SiddhiAppRuntime`: Lifecycle methods (`start`, `shutdown`) are very basic. `persist`, `restore` not implemented.
    *   Error handling throughout `siddhi-core` is minimal.
*   **Extensions Framework**:
    *   `ScalarFunctionExecutor` allows registering stateful user-defined functions.
    *   Placeholders for other extension types (Window, Sink, Source, Store, Mapper, AttributeAggregator, Script) are largely missing.
*   **DataSources**: `DataSource` trait is a placeholder. No actual implementations or integration with table stores.
*   **Concurrency**: While `Arc<Mutex<T>>` is used in places, detailed analysis and implementation of Siddhi's concurrency model (thread pools for async junctions, partitioned execution) are pending.

## Testing Status

*   **`query_api`**: Basic unit tests for constructors and getters of key data structures are planned / partially implemented.
*   **`siddhi-core`**: Some unit tests for basic expression executors are planned / partially implemented.
*   **Integration Testing**: A test case for a simple filter/projection query (`FROM InputStream[filter] SELECT ... INSERT INTO OutputStream`) has been outlined. This test was used to conceptually verify the design of Phase 1 components. **Actual execution and passing of this test requires further implementation and debugging.**
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

## Next Planned Phases (High-Level)

1.  **Stabilize Phase 1**: Make the `test_simple_filter_projection_query` compile and run successfully by fully implementing the simplified logic paths in `ExpressionParser`, `VariableExpressionExecutor`, `FilterProcessor`, `SelectProcessor`, and event data handling.
2.  **Basic Stateful Operations**: Introduce `LengthWindowProcessor` and other simple stateful processors.
3.  **Expand Core Logic**: Gradually implement more expression executors, stream processors, join capabilities, and aggregation functions.
4.  **SiddhiQL Parsing**: Integrate a proper SiddhiQL parser (potentially by exploring options like FFI to the Java ANTLR parser, or using a Rust parsing library for a subset of SiddhiQL).

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
