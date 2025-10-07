// SPDX-License-Identifier: MIT OR Apache-2.0

//! SQL Preprocessor - Extract Streaming Extensions
//!
//! Preprocesses SQL to extract streaming-specific clauses (like WINDOW)
//! before standard SQL parsing.

use once_cell::sync::Lazy;
use regex::Regex;
use sqlparser::ast::Expr as SqlExpr;

use super::error::PreprocessorError;

/// Regex to match WINDOW clause in SQL
static WINDOW_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bWINDOW\s+(TUMBLING|SLIDING|LENGTH|SESSION)\s*\(([^)]+)\)")
        .expect("Invalid WINDOW regex")
});

/// Time unit for window specifications
#[derive(Debug, Clone, PartialEq)]
pub enum TimeUnit {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
}

/// Window specification extracted from SQL
#[derive(Debug, Clone, PartialEq)]
pub enum WindowSpec {
    Tumbling {
        value: i64,
        unit: TimeUnit,
    },
    Sliding {
        window_value: i64,
        window_unit: TimeUnit,
        slide_value: i64,
        slide_unit: TimeUnit,
    },
    Length {
        size: i64,
    },
    Session {
        value: i64,
        unit: TimeUnit,
    },
}

/// Text representation of window clause
#[derive(Debug, Clone)]
pub struct WindowClauseText {
    pub full_match: String,
    pub window_type: String,
    pub parameters: String,
    pub spec: WindowSpec,
}

/// Preprocessed SQL with extracted components
#[derive(Debug, Clone)]
pub struct PreprocessedSql {
    pub standard_sql: String,
    pub window_clause: Option<WindowClauseText>,
}

/// SQL Preprocessor
pub struct SqlPreprocessor;

impl SqlPreprocessor {
    /// Preprocess SQL to extract streaming extensions
    pub fn preprocess(sql: &str) -> Result<PreprocessedSql, PreprocessorError> {
        let mut result = PreprocessedSql {
            standard_sql: sql.to_string(),
            window_clause: None,
        };

        // Extract WINDOW clause if present
        if let Some(captures) = WINDOW_REGEX.captures(sql) {
            let full_match = captures
                .get(0)
                .ok_or_else(|| PreprocessorError::WindowParseFailed("No match found".to_string()))?
                .as_str()
                .to_string();

            let window_type = captures
                .get(1)
                .ok_or_else(|| PreprocessorError::WindowParseFailed("No window type".to_string()))?
                .as_str()
                .to_uppercase();

            let parameters = captures
                .get(2)
                .ok_or_else(|| PreprocessorError::WindowParseFailed("No parameters".to_string()))?
                .as_str()
                .trim()
                .to_string();

            // Parse window specification
            let spec = Self::parse_window_spec(&window_type, &parameters)?;

            result.window_clause = Some(WindowClauseText {
                full_match: full_match.clone(),
                window_type,
                parameters,
                spec,
            });

            // Remove WINDOW clause from SQL
            result.standard_sql = sql.replace(&full_match, "").trim().to_string();
        }

        Ok(result)
    }

    /// Parse window specification from type and parameters
    fn parse_window_spec(window_type: &str, params: &str) -> Result<WindowSpec, PreprocessorError> {
        match window_type {
            "TUMBLING" => {
                let (value, unit) = Self::parse_time_param(params)?;
                Ok(WindowSpec::Tumbling { value, unit })
            }
            "SLIDING" => {
                let parts: Vec<&str> = params.split(',').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    return Err(PreprocessorError::InvalidWindowParams(
                        "SLIDING window requires 2 parameters (window, slide)".to_string(),
                    ));
                }
                let (window_value, window_unit) = Self::parse_time_param(parts[0])?;
                let (slide_value, slide_unit) = Self::parse_time_param(parts[1])?;
                Ok(WindowSpec::Sliding {
                    window_value,
                    window_unit,
                    slide_value,
                    slide_unit,
                })
            }
            "LENGTH" => {
                let size = Self::parse_int_param(params)?;
                Ok(WindowSpec::Length { size })
            }
            "SESSION" => {
                let (value, unit) = Self::parse_time_param(params)?;
                Ok(WindowSpec::Session { value, unit })
            }
            _ => Err(PreprocessorError::InvalidWindowType(
                window_type.to_string(),
            )),
        }
    }

    /// Parse time parameter (supports seconds, milliseconds, etc.)
    /// Returns (value, unit) tuple
    fn parse_time_param(param: &str) -> Result<(i64, TimeUnit), PreprocessorError> {
        let param = param.trim();

        // Handle INTERVAL syntax: INTERVAL '5' SECOND
        if param.to_uppercase().starts_with("INTERVAL") {
            let parts: Vec<&str> = param.split_whitespace().collect();
            if parts.len() >= 3 {
                let value_str = parts[1].trim_matches('\'').trim_matches('"');
                let unit_str = parts[2].to_uppercase();
                let value: i64 = value_str.parse().map_err(|_| {
                    PreprocessorError::InvalidWindowParams(format!("Invalid number: {}", value_str))
                })?;

                let unit = match unit_str.as_str() {
                    "MILLISECOND" | "MILLISECONDS" => TimeUnit::Milliseconds,
                    "SECOND" | "SECONDS" => TimeUnit::Seconds,
                    "MINUTE" | "MINUTES" => TimeUnit::Minutes,
                    "HOUR" | "HOURS" => TimeUnit::Hours,
                    _ => {
                        return Err(PreprocessorError::InvalidWindowParams(format!(
                            "Unknown time unit: {}",
                            unit_str
                        )))
                    }
                };

                return Ok((value, unit));
            }
        }

        // Handle direct numeric values (assume milliseconds)
        if let Ok(num) = param.parse::<i64>() {
            return Ok((num, TimeUnit::Milliseconds));
        }

        Err(PreprocessorError::InvalidWindowParams(format!(
            "Cannot parse time: {}",
            param
        )))
    }

    /// Parse integer parameter
    fn parse_int_param(param: &str) -> Result<i64, PreprocessorError> {
        param.trim().parse::<i64>().map_err(|_| {
            PreprocessorError::InvalidWindowParams(format!("Invalid integer: {}", param))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_no_window() {
        let sql = "SELECT * FROM stream";
        let result = SqlPreprocessor::preprocess(sql).unwrap();
        assert_eq!(result.standard_sql, sql);
        assert!(result.window_clause.is_none());
    }

    #[test]
    fn test_preprocess_tumbling_window() {
        let sql = "SELECT * FROM stream WINDOW TUMBLING(INTERVAL '5' SECOND)";
        let result = SqlPreprocessor::preprocess(sql).unwrap();

        assert_eq!(result.standard_sql, "SELECT * FROM stream");
        assert!(result.window_clause.is_some());

        let window = result.window_clause.unwrap();
        assert_eq!(window.window_type, "TUMBLING");
        assert_eq!(
            window.spec,
            WindowSpec::Tumbling {
                value: 5,
                unit: TimeUnit::Seconds
            }
        );
    }

    #[test]
    fn test_preprocess_sliding_window() {
        let sql = "SELECT * FROM stream WINDOW SLIDING(INTERVAL '10' SECOND, INTERVAL '5' SECOND)";
        let result = SqlPreprocessor::preprocess(sql).unwrap();

        let window = result.window_clause.unwrap();
        assert_eq!(window.window_type, "SLIDING");
        assert_eq!(
            window.spec,
            WindowSpec::Sliding {
                window_value: 10,
                window_unit: TimeUnit::Seconds,
                slide_value: 5,
                slide_unit: TimeUnit::Seconds
            }
        );
    }

    #[test]
    fn test_preprocess_length_window() {
        let sql = "SELECT * FROM stream WINDOW LENGTH(100)";
        let result = SqlPreprocessor::preprocess(sql).unwrap();

        let window = result.window_clause.unwrap();
        assert_eq!(window.window_type, "LENGTH");
        assert_eq!(window.spec, WindowSpec::Length { size: 100 });
    }

    #[test]
    fn test_preprocess_session_window() {
        let sql = "SELECT * FROM stream WINDOW SESSION(INTERVAL '30' SECOND)";
        let result = SqlPreprocessor::preprocess(sql).unwrap();

        let window = result.window_clause.unwrap();
        assert_eq!(window.window_type, "SESSION");
        assert_eq!(
            window.spec,
            WindowSpec::Session {
                value: 30,
                unit: TimeUnit::Seconds
            }
        );
    }

    #[test]
    fn test_parse_time_milliseconds() {
        let result = SqlPreprocessor::parse_time_param("INTERVAL '100' MILLISECOND").unwrap();
        assert_eq!(result, (100, TimeUnit::Milliseconds));
    }

    #[test]
    fn test_parse_time_seconds() {
        let result = SqlPreprocessor::parse_time_param("INTERVAL '5' SECOND").unwrap();
        assert_eq!(result, (5, TimeUnit::Seconds));
    }

    #[test]
    fn test_parse_time_minutes() {
        let result = SqlPreprocessor::parse_time_param("INTERVAL '2' MINUTE").unwrap();
        assert_eq!(result, (2, TimeUnit::Minutes));
    }

    #[test]
    fn test_parse_direct_number() {
        let result = SqlPreprocessor::parse_time_param("5000").unwrap();
        assert_eq!(result, (5000, TimeUnit::Milliseconds));
    }

    #[test]
    fn test_case_insensitive_window_type() {
        let sql = "SELECT * FROM stream window tumbling(INTERVAL '5' SECOND)";
        let result = SqlPreprocessor::preprocess(sql).unwrap();
        assert!(result.window_clause.is_some());
    }
}
