# `core::util`

A partial port of the `io.eventflux.core.util` Java package providing the
utility functions required by the Rust runtime.  Additional helpers can
be added as needed.

## Implemented Modules

* `executor_service` – lightweight wrapper around a Rayon thread pool
  providing an `ExecutorService` style API.
* `attribute_converter` – helper functions for converting generic values
  to `AttributeValue` according to an `Attribute::Type`.
* `id_generator` – simple monotonically increasing id generator.
* `metrics` – simple in-memory metrics trackers for latency, throughput
  and buffered event counts.
* `parser` – runtime parsers used by the experimental engine.
* `eventflux_constants` – constants shared across the core modules.
* `scheduler` – lightweight task scheduler for time based windows and triggers.

Metrics tracking is intentionally lightweight but used throughout the
runtime.

Contributions welcome!
