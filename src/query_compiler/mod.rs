pub mod siddhi_compiler;
// pub mod error; // If specific compiler errors are defined later

// Re-export specific public functions from siddhi_compiler.rs module
pub use self::siddhi_compiler::{
    parse,
    parse_stream_definition,
    parse_table_definition,
    parse_aggregation_definition,
    parse_partition,
    parse_query,
    parse_function_definition,
    parse_time_constant,
    parse_on_demand_query,
    parse_store_query,
    parse_expression,
    update_variables, // if this is also intended to be part of the public API
};

// Or, if we prefer a struct-based approach for the compiler in Rust eventually:
// pub use self::siddhi_compiler::SiddhiCompiler;
// And then functions would be SiddhiCompiler::parse(), etc.
// For now, free functions are used as per the direct translation of static Java methods.
