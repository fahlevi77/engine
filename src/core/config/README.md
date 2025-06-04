# Configuration Module (`core::config`)

This directory contains Rust ports of the Java `io.siddhi.core.config` package.  
The goal of these implementations is to mirror the behaviour of the Java classes
while allowing the rest of the Rust code base to compile.  Many complex
subsystems of Siddhi (persistence stores, metrics, snapshotting, etc.) are not
yet implemented in Rust.  Placeholder structs and simple `String` types are used
in their place so that APIs remain compatible.

## Files

- `statistics_configuration.rs` – Holds configuration for statistics/metrics.
- `siddhi_context.rs` – Global context shared among Siddhi applications.
- `siddhi_app_context.rs` – Context specific to a single Siddhi application.
- `siddhi_query_context.rs` – Context associated with individual queries.
- `siddhi_on_demand_query_context.rs` – Context for on-demand queries.

## Notes / TODO

- Extension holders, persistence stores, and error stores are represented with
  simple placeholders.  Actual implementations should provide concrete traits or
  structs.
- `SiddhiAppContext::generate_state_holder` and
  `SiddhiQueryContext::generate_state_holder` require snapshot storage logic and
  are left as future work.
- Exception handling for disruptor and runtime errors is simplified to string
  placeholders.

These modules should compile and provide the same public API surface as their
Java counterparts, enabling further work on the Siddhi Rust port.
