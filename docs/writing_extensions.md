# Writing Extensions

This guide explains how to extend the Siddhi Rust runtime with custom
components.  Extensions are registered with a `SiddhiManager` and become
available to applications via annotations like `@source`, `@sink` or `@store`.

## Table Extensions

A table extension implements the `Table` trait and provides a corresponding
`TableFactory`.  The factory is registered under a unique name:

```rust
use siddhi_rust::core::extension::TableFactory;
use siddhi_rust::core::table::Table;

pub struct MyTable;
impl Table for MyTable { /* ... */ }

#[derive(Clone)]
pub struct MyTableFactory;
impl TableFactory for MyTableFactory {
    fn name(&self) -> &'static str { "myStore" }
    fn create(
        &self,
        name: String,
        props: std::collections::HashMap<String, String>,
        ctx: std::sync::Arc<siddhi_rust::core::config::siddhi_context::SiddhiContext>,
    ) -> Result<std::sync::Arc<dyn Table>, String> {
        Ok(std::sync::Arc::new(MyTable))
    }
    fn clone_box(&self) -> Box<dyn TableFactory> { Box::new(self.clone()) }
}
```

After registering the factory with a `SiddhiManager`, applications can define a
table using `@store(type='myStore')`.

### Compiled Conditions

To execute query conditions efficiently, tables can translate expressions into a
custom representation by implementing `compile_condition` and
`compile_update_set`.  The returned structures implement the marker traits
`CompiledCondition` and `CompiledUpdateSet` which the runtime passes back to the
same table instance for `find`, `update` and `delete` operations.

A table may also implement `compile_join_condition` to optimise stream-table
joins.  The provided `JdbcTable` demonstrates translating expressions into SQL
for execution inside a database, while `CacheTable` performs in-memory matching
using compiled values.

Extensions for other storage engines can follow these examples to provide their
own compiled condition formats.
