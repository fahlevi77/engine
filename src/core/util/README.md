# `core::util`

A partial port of the `io.siddhi.core.util` Java package.  Only the
components currently needed by the Rust prototype are implemented.
Future ports should extend this module structure and remove the
placeholders noted below.

## Implemented Modules

* `executor_service` – lightweight wrapper around a Rayon thread pool
  providing an `ExecutorService` style API.
* `attribute_converter` – helper functions for converting generic values
  to `AttributeValue` according to an `Attribute::Type`.
* `id_generator` – simple monotonically increasing id generator.
* `metrics_placeholders` – stub types for metrics tracking.
* `parser` – runtime parsers used by the experimental engine.
* `siddhi_constants` – constants shared across the core modules.
* `scheduler` – lightweight task scheduler for time based windows and triggers.

## TODOs

* Many Java utility classes (lock helpers, snapshot utilities,
  etc.) are still missing.
* Metrics trackers should be replaced with real implementations.

Contributions welcome!
