# Stream Module

This module contains the Rust translations of Siddhi's core stream package.
The focus is on providing a minimal, but faithful, port of the Java
implementation.  Many advanced features such as metrics, error stores and
thread barriers are represented by placeholders.

## Key Components

* `StreamJunction` – routes events between publishers and subscribers.  Supports
  optional asynchronous processing using a bounded channel.
* `Publisher` – equivalent to the Java inner class. Implements the
  `InputProcessor` trait so it can be used by the input subsystem.
* `InputHandler`, `InputManager`, `InputDistributor` and `InputEntryValve` –
  provide the event injection path into Siddhi.  They mirror the original Java
  classes in structure, though many behaviours are simplified.
* `ServiceDeploymentInfo` – basic representation of service deployment metadata.

## Notes / TODO

* Event cloning for multiple subscribers is currently naive.  Events are passed
  to the first subscriber only.  A proper event chunk cloning or pooling
  mechanism is required.
* Metrics, fault stream handling and error storage are largely unimplemented.
* A real `ThreadBarrier` implementation is needed for accurate entry valve
  behaviour.
