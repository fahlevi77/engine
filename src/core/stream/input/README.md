# Input Submodule

Rust port of EventFlux's `io.eventflux.core.stream.input` package.  The
implementation mirrors the Java classes where possible but remains
simplified.

## Notes

* `InputHandler`, `InputManager`, `InputDistributor` and
  `InputEntryValve` follow the same responsibilities as in the Java
  implementation.  Thread barriers and metrics are represented by
  placeholders.
* `TableInputHandler` allows events to be inserted into tables so they can be
  joined with streams.
* The event injection API is functional for the basic stateless query
  paths used in the current integration tests.

