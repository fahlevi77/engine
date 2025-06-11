# State Event Module (`core::event::state`)

Rust translation of Siddhi's `io.siddhi.core.event.state` package.  Only the
portion required for the current prototype is implemented.

## Implemented Components

- `StateEvent` – internal event type used for joins and patterns.  Supports
  attribute access via Siddhi's position arrays and linked list chaining.
- `MetaStateEvent` – holds the meta information about a `StateEvent` such as the
  underlying `MetaStreamEvent`s and output mappings.
- `MetaStateEventAttribute` – mapping of a single output attribute to its
  position within a `StateEvent`.
- `StateEventFactory` – helper for constructing `StateEvent` instances based on
  meta information.
- `StateEventCloner` – shallow cloning utility for `StateEvent` used by stateful
  processors.
- *Populator* utilities (`StateEventPopulator`, etc.) – used to copy attributes
  between events when constructing `StateEvent` instances.
