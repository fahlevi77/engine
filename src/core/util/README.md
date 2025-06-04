# `core::util`

A partial port of the `io.siddhi.core.util` Java package.  Only the
components currently needed by the Rust prototype are implemented.
Future ports should extend this module structure and remove the
placeholders noted below.

## Implemented Modules

* `executor_service` – a very small thread pool acting as a stand in for
  Java's `ExecutorService`.
* `attribute_converter` – helper functions for converting generic values
  to `AttributeValue` according to an `Attribute::Type`.
* `id_generator` – simple monotonically increasing id generator.
* `metrics_placeholders` – stub types for metrics tracking.
* `parser` – runtime parsers used by the experimental engine.
* `siddhi_constants` – constants shared across the core modules.

## TODOs

* Many Java utility classes (lock helpers, snapshot utilities,
  scheduler, etc.) are still missing.
* Metrics trackers should be replaced with real implementations.
* The executor service is intentionally small and should be replaced by
  a robust async/thread‑pool library when the runtime requires it.

Contributions welcome!
