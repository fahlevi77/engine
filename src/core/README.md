# Siddhi Core (Rust Port)

This directory mirrors the `io.siddhi.core` package from the Java
implementation.  Only a minimal subset of the original runtime is
implemented.  The goal of the current port is to support a very simple
stateless query pipeline so the integration test in `src/lib.rs`
(`test_simple_filter_projection_query`) executes end-to-end.

## Notes

* `stream_junction::send_event` now converts incoming `Event` objects
  into `StreamEvent`s with the incoming data placed in the
  `before_window_data` array.  This allows `FilterProcessor` and
  `SelectProcessor` to access attributes correctly.
* Large parts of the original Siddhi runtime remain unimplemented
  (windows, tables, persistence, etc.).  Placeholders are provided where
  required by the API.

