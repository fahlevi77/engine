# Query Module

This module contains the Rust translation of Siddhi's core `query` package.  It implements
runtime structures that execute a single Siddhi query as well as the processors that
form the internal processing chain.

## Key Components

* `QueryRuntime` – lightweight representation of a query at runtime.  Holds the query
  name, optional API model and the head of the processor chain.  It implements the
  `QueryRuntimeTrait` which mirrors the minimal Java `QueryRuntime` interface.
* `processor` – currently includes `FilterProcessor` as an example stream processor.
* `selector` – provides `SelectProcessor` and `OutputAttributeProcessor` for handling
  `select` clauses.
* `output` – contains terminal processors such as `InsertIntoStreamProcessor` and
  `CallbackProcessor` that publish results.

## Notes / TODO

* Many advanced Siddhi features like windows, joins and rate limiting are not yet
  ported.  Placeholders are left where appropriate.
* `QueryRuntime` does not implement snapshotting or state management.  These should
  be added when the relevant subsystems are available.
* Event chunk manipulation in `FilterProcessor` and `SelectProcessor` is simplified
  and should be revisited for efficiency.

