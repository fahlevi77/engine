# Stream Event Module (`core::event::stream`)

Rust equivalent of the Java `io.siddhi.core.event.stream` package.

## Implemented Components

- `StreamEvent` – main internal event representation used during stream
  processing.
- `MetaStreamEvent` – describes the structure of `StreamEvent` instances,
  including before/after window data and output data.
- `StreamEventFactory` – convenience for creating `StreamEvent` objects based on
  meta information.
- `Operation` – small helper struct mirroring Siddhi's `Operation` class.

## Missing Parts / TODO

Several advanced helpers (`StreamEventCloner`, converters, holders and
populators) have not yet been ported.  Serialization helpers present in the Java
implementation are also omitted for now.
