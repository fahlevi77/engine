# Configuration Module (`core::config`)

This directory contains Rust ports of the Java `io.siddhi.core.config` package.  
The goal of these implementations is to mirror the behaviour of the Java classes
while allowing the rest of the Rust code base to compile.  Persistence stores
(in-memory, file and SQLite) and basic metrics are available, while other
subsystems still rely on simplified placeholders.

## Files

- `statistics_configuration.rs` – Holds configuration for statistics/metrics.
- `siddhi_context.rs` – Global context shared among Siddhi applications.
- `siddhi_app_context.rs` – Context specific to a single Siddhi application.
- `siddhi_query_context.rs` – Context associated with individual queries.
- `siddhi_on_demand_query_context.rs` – Context for on-demand queries.

## Notes

 - Extension holders and error stores are represented with simple placeholders.
 - `SiddhiAppContext::generate_state_holder` and
   `SiddhiQueryContext::generate_state_holder` will evolve with the
   snapshotting system.
- Exception handling for disruptor and runtime errors is simplified to string
  placeholders.

These modules should compile and provide the same public API surface as their
Java counterparts, enabling further work on the Siddhi Rust port.
