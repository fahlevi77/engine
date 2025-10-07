# Output Submodule

Partial port of EventFlux's `io.eventflux.core.stream.output` package.
`StreamCallback` trait models the Java abstract class and a simple
`LogStreamCallback` implementation is provided for tests.

## Notes

* Only callback based output is currently supported.  Sink adapters and
  mappers are not yet implemented.
* `to_map` helpers mimic the Java convenience methods for converting an
  `Event` into a `HashMap` when a stream definition is available.

