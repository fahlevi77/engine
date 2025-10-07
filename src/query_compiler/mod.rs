// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod eventflux_compiler;
// pub mod error; // If specific compiler errors are defined later

// Re-export specific public functions from eventflux_compiler.rs module
pub use self::eventflux_compiler::{
    parse,
    parse_aggregation_definition,
    parse_expression,
    parse_function_definition,
    parse_on_demand_query,
    parse_partition,
    parse_query,
    parse_set_clause,
    parse_store_query,
    parse_stream_definition,
    parse_table_definition,
    parse_time_constant,
    parse_trigger_definition,
    parse_window_definition,
    update_variables, // if this is also intended to be part of the public API
};

// Or, if we prefer a struct-based approach for the compiler in Rust eventually:
// pub use self::eventflux_compiler::EventFluxCompiler;
// And then functions would be EventFluxCompiler::parse(), etc.
// For now, free functions are used as per the direct translation of static Java methods.
