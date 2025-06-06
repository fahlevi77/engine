# Input Submodule

Rust port of Siddhi's `io.siddhi.core.stream.input` package.  The
implementation mirrors the Java classes where possible but remains
simplified.

## Notes

* `InputHandler`, `InputManager`, `InputDistributor` and
  `InputEntryValve` follow the same responsibilities as in the Java
  implementation.  Thread barriers and metrics are represented by
  placeholders.
* `TableInputHandler` is largely unimplemented because table support is
  missing in this prototype.
* The event injection API is functional for the basic stateless query
  paths used in the current integration tests.

