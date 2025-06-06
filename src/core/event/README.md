# Event Module (`core::event`)

This directory contains the Rust translation of the Java
`io.siddhi.core.event` package.  The structures mirror the original
implementation as closely as possible while adopting idiomatic Rust
where appropriate.

## Highlights

- `value.rs` provides `AttributeValue` representing the different types
  that can be carried inside an event.
- `event.rs`, `stream_event.rs` and `state_event.rs` implement the core
  event types used by Siddhi.  Methods such as `copy_from_complex`,
  attribute access via Siddhi position arrays and event chain
  manipulation are supported.
- `complex_event.rs` defines the `ComplexEvent` trait and the
  `ComplexEventType` enum.
- `meta_state_event.rs` and `meta_stream_event.rs` hold meta information
  describing the structure of state and stream events.
- Constants used for array index calculations are located in
  `core::util::siddhi_constants.rs`.

## TODO

The current implementation is functional enough for simple query
execution but several areas still need work:

- Comprehensive cloning of event chains.
- Full parity with all methods of the Java classes (some advanced
  serialization helpers are omitted).
- Additional utility types under `state` and `stream` packages are only
  partially ported.  Basic factories and attribute mapping structures are
  available but cloners and populaters are TODO.

Contributions and further improvements are welcome.
