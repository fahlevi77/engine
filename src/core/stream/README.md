# Stream Module

This module contains the Rust translations of EventFlux's core stream package.
The focus is on providing a minimal, but faithful, port of the Java
implementation.  Many advanced features such as metrics, error stores and
thread barriers are represented by placeholders.

## Key Components

* `StreamJunction` – routes events between publishers and subscribers.  Supports
  optional asynchronous processing using a bounded channel.
  `subscribe` and `unsubscribe` mirror the Java methods for managing
  downstream processors.  `start_processing`/`stop_processing` start or
  shut down the internal async task.
* `Publisher` – equivalent to the Java inner class. Implements the
  `InputProcessor` trait so it can be used by the input subsystem.
* `InputHandler`, `InputManager`, `InputDistributor` and `InputEntryValve` –
  provide the event injection path into EventFlux.  They mirror the original Java
  classes in structure, though many behaviours are simplified.
* `ServiceDeploymentInfo` – basic representation of service deployment metadata.

## Notes

* Event chunks are now cloned for each subscriber and dispatched using the
  improved executor service when a junction is marked asynchronous.
* Basic metrics tracking and fault stream/error store routing are now
  implemented.
* `ThreadBarrier` now coordinates event flow in `InputEntryValve` to maintain
  ordering when required.
