//! SQL Compiler Module
//!
//! This module provides SQL parsing and compilation capabilities for EventFlux Rust.
//! It converts SQL syntax into EventFlux's query_api structures for execution.
//!
//! # Architecture
//!
//! The SQL compiler follows a multi-stage pipeline:
//! 1. **Preprocessing** - Extract streaming extensions (WINDOW clauses)
//! 2. **DDL Parsing** - Parse CREATE STREAM statements
//! 3. **SQL Parsing** - Use sqlparser-rs for standard SQL
//! 4. **Type Mapping** - Convert SQL types to AttributeType
//! 5. **SELECT Expansion** - Expand SELECT * using schema
//! 6. **Conversion** - Convert to query_api::Query structures
//!
//! # Example
//!
//! ```rust,ignore
//! use eventflux_rust::sql_compiler::parse_sql_application;
//!
//! let sql = r#"
//!     CREATE STREAM StockStream (symbol STRING, price DOUBLE);
//!
//!     SELECT symbol, price
//!     FROM StockStream
//!     WHERE price > 100;
//! "#;
//!
//! let app = parse_sql_application(sql)?;
//! ```

pub mod application;
pub mod catalog;
pub mod converter;
pub mod ddl;
pub mod error;
pub mod expansion;
pub mod preprocessor;
pub mod type_mapping;

// Re-export main types for convenient access
pub use application::parse_sql_application;
pub use catalog::{SqlApplication, SqlCatalog};
pub use converter::SqlConverter;
pub use ddl::{CreateStreamInfo, DdlParser};
pub use error::{
    ApplicationError, ConverterError, DdlError, ExpansionError, PreprocessorError,
    SqlCompilerError, TypeError,
};
pub use expansion::SelectExpander;
pub use preprocessor::{PreprocessedSql, SqlPreprocessor, WindowClauseText};
pub use type_mapping::{attribute_type_to_sql_type, sql_type_to_attribute_type};

/// Parse a complete SQL application with multiple statements
pub fn parse(sql: &str) -> Result<SqlApplication, SqlCompilerError> {
    parse_sql_application(sql).map_err(SqlCompilerError::from)
}

/// Level 1 API: Compile a single SQL query with a given catalog
///
/// This is the low-level API for converting a single SQL query to a query_api::Query.
/// Use this when you manually manage the catalog and need fine-grained control.
///
/// # Example
/// ```rust,ignore
/// let mut catalog = SqlCatalog::new();
/// catalog.register_stream("StockStream".to_string(), stream_def)?;
///
/// let query = compile_sql_query(
///     "SELECT symbol, price FROM StockStream WHERE price > 100",
///     &catalog
/// )?;
/// ```
pub fn compile_sql_query(
    sql: &str,
    catalog: &SqlCatalog,
) -> Result<crate::query_api::execution::query::Query, SqlCompilerError> {
    SqlConverter::convert(sql, catalog).map_err(SqlCompilerError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all types are accessible
        let _catalog = SqlCatalog::new();
    }
}
