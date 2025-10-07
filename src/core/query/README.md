# Query Module

This module contains the Rust translation of EventFlux's core `query` package.  It implements
runtime structures that execute a single EventFlux query as well as the processors that
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

## Notes

The module now supports windows, joins and pattern execution.  Snapshotting of
stateful processors is handled via the `SnapshotService`.  Event chunk
manipulation in `FilterProcessor` and `SelectProcessor` remains simplified but
works for the common query flows.

