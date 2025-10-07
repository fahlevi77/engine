# Test Utilities

The tests in this crate rely on a small helper called `AppRunner` located in
`tests/common/mod.rs`.  `AppRunner` parses a EventFlux application string using the
embedded EventFluxQL grammar and creates a `EventFluxAppRuntime` through a
`EventFluxManager`.  It provides convenience methods to feed events into input
streams and collects events emitted from an output stream for assertions.

Usage pattern:

```rust
let app = "define stream In (a int); define stream Out (a int); \n \
            from In select a insert into Out;";
let runner = AppRunner::new(app, "Out");
runner.send("In", vec![AttributeValue::Int(42)]);
let out = runner.shutdown();
assert_eq!(out, vec![vec![AttributeValue::Int(42)]]);
```

Additional test modules can include `#[path = "common/mod.rs"] mod common;` and
use `common::AppRunner` as above.
