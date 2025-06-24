# Executor Module

This directory contains Rust implementations of Siddhi's `executor` package.  The
code mirrors the Java classes where practical.  Some functionality has been
simplified to keep the initial port manageable.

## Newly Added

* **EventVariableFunctionExecutor** – retrieves a `StreamEvent` from a
  `StateEvent` and returns it as an OBJECT value.  The returned event is a
  shallow clone without the `next` link.
* **MultiValueVariableFunctionExecutor** – collects attribute values from all
  events in a `StateEvent` chain and returns them as a vector wrapped in an
  OBJECT value.
* Added `clone_without_next` helper on `StreamEvent` used by the new executors.
* Initial set of incremental aggregation executors (`timestampInMilliseconds`,
  `getTimeZone`, `shouldUpdate`, `getAggregationStartTime`,
  `getStartTimeEndTime`) have been ported from the Java implementation.

