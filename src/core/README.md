# EventFlux Core (Rust Port)

This directory mirrors the `io.eventflux.core` package from the Java
implementation.  The Rust port now supports windows, joins, patterns,
tables and persistence, providing a functional runtime for many EventFlux
applications.

## Notes

* `stream_junction::send_event` now converts incoming `Event` objects
  into `StreamEvent`s with the incoming data placed in the
  `before_window_data` array.  This allows `FilterProcessor` and
  `SelectProcessor` to access attributes correctly.
* Core processors for windows, joins and patterns are implemented and
  tables can be queried or joined using the in-memory and JDBC stores.
  Persistence is available via in-memory, file and SQLite stores.

