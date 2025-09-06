// Note: query_api types are used for return types.
// Ensure these paths are correct based on query_api module structure and re-exports.
use crate::query_api::{
    definition::{
        attribute::Type as AttributeType, AggregationDefinition, FunctionDefinition,
        StreamDefinition, TableDefinition, TriggerDefinition, WindowDefinition,
    },
    execution::{
        query::{output::stream::UpdateSet, OnDemandQuery, Query, StoreQuery},
        ExecutionElement, Partition,
    },
    expression::{constant::Constant as ExpressionConstant, Expression},
    SiddhiApp,
};
use lalrpop_util::lalrpop_mod;
use once_cell::sync::Lazy;
use regex::Regex;
use std::env;

lalrpop_mod!(pub grammar, "/query_compiler/grammar.rs");

// Cache for commonly used regex patterns
static VAR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\{(\w+)\}").expect("Invalid variable pattern")
});

// SiddhiCompiler in Java has only static methods.
// In Rust, these are translated as free functions within this module.

// update_variables function (ported from Java SiddhiCompiler)
// This function needs to be pub if called directly from outside, or pub(crate) if only by parse functions here.
// The prompt implies it's called by the parse functions.
// Making it public as per later interpretation of prompt.
pub fn update_variables(siddhi_app_string: &str) -> Result<String, String> {
    if !siddhi_app_string.contains('$') {
        return Ok(siddhi_app_string.to_string());
    }

    let mut updated_siddhi_app = String::with_capacity(siddhi_app_string.len());
    let mut last_match_end = 0;

    for captures in VAR_PATTERN.captures_iter(siddhi_app_string) {
        let full_match = captures.get(0).unwrap(); // The whole ${varName}
        let var_name = captures.get(1).unwrap().as_str();

        updated_siddhi_app.push_str(&siddhi_app_string[last_match_end..full_match.start()]);

        match env::var(var_name) {
            Ok(value) => updated_siddhi_app.push_str(&value),
            Err(_) => {
                // Enhanced error reporting with position information
                let line_pos = calculate_line_position(&siddhi_app_string, full_match.start());
                return Err(format!(
                    "No system or environmental variable found for '${{{var_name}}}' at line {} column {}",
                    line_pos.0, line_pos.1
                ));
            }
        }
        last_match_end = full_match.end();
    }
    updated_siddhi_app.push_str(&siddhi_app_string[last_match_end..]);
    Ok(updated_siddhi_app)
}

// Helper function to calculate line and column position
fn calculate_line_position(text: &str, position: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    
    for (i, ch) in text.char_indices() {
        if i >= position {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    
    (line, column)
}

/// Strips SQL-style comments from Siddhi query string
/// Handles both single-line (--) and multi-line (/* */) comments  
/// Preserves strings to avoid removing comment-like content within them
fn strip_comments(siddhi_app_string: &str) -> String {
    let mut result = String::with_capacity(siddhi_app_string.len());
    let mut chars = siddhi_app_string.chars().peekable();
    let mut in_string = false;
    let mut string_quote = ' ';
    
    while let Some(ch) = chars.next() {
        // Handle string literals to preserve content
        if !in_string && (ch == '\'' || ch == '"') {
            in_string = true;
            string_quote = ch;
            result.push(ch);
        } else if in_string && ch == string_quote {
            // Check for escaped quotes
            if chars.peek() == Some(&string_quote) {
                result.push(ch);
                result.push(chars.next().unwrap());
            } else {
                in_string = false;
                result.push(ch);
            }
        } else if in_string {
            result.push(ch);
        } else if ch == '-' && chars.peek() == Some(&'-') {
            // SQL-style single-line comment
            chars.next(); // consume second '-'
            // Skip until end of line
            while let Some(c) = chars.next() {
                if c == '\n' {
                    result.push('\n'); // Preserve line breaks for error reporting
                    break;
                }
            }
        } else if ch == '/' && chars.peek() == Some(&'*') {
            // Multi-line comment
            chars.next(); // consume '*'
            let mut prev = ' ';
            while let Some(c) = chars.next() {
                if prev == '*' && c == '/' {
                    break;
                }
                if c == '\n' {
                    result.push('\n'); // Preserve line breaks
                }
                prev = c;
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Preprocesses Siddhi app string to handle various syntax improvements
/// - Strips comments
/// - Normalizes whitespace
/// - Prepares for parsing
fn preprocess_siddhi_app(siddhi_app_string: &str) -> String {
    // First strip comments
    let without_comments = strip_comments(siddhi_app_string);
    
    // Normalize consecutive whitespace while preserving structure
    let lines: Vec<String> = without_comments
        .lines()
        .map(|line| {
            // Trim trailing whitespace but preserve leading for structure
            line.trim_end().to_string()
        })
        .collect();
    
    lines.join("\n")
}

fn parse_attribute_type(t: &str) -> Result<AttributeType, String> {
    match t.to_lowercase().as_str() {
        "string" => Ok(AttributeType::STRING),
        "int" => Ok(AttributeType::INT),
        "long" => Ok(AttributeType::LONG),
        "float" => Ok(AttributeType::FLOAT),
        "double" => Ok(AttributeType::DOUBLE),
        "bool" | "boolean" => Ok(AttributeType::BOOL),
        "object" => Ok(AttributeType::OBJECT),
        _ => Err(format!("Unknown attribute type: {t}")),
    }
}

pub fn parse(siddhi_app_string: &str) -> Result<SiddhiApp, String> {
    // Preprocess to handle comments and syntax normalization
    let preprocessed = preprocess_siddhi_app(siddhi_app_string);
    let s = update_variables(&preprocessed)?;

    let mut annotations = Vec::new();
    let mut lines_without_ann = Vec::with_capacity(s.lines().count());
    
    // Create parser once for reuse
    let ann_parser = grammar::AnnotationStmtParser::new();
    
    for line in s.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('@') {
            if let Ok(ann) = ann_parser.parse(trimmed) {
                if ann.name.eq_ignore_ascii_case("app") {
                    annotations.push(ann);
                    continue;
                }
            }
        }
        lines_without_ann.push(line);
    }
    let s_no_ann = lines_without_ann.join("\n");

    let app_name = annotations
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case("app"))
        .and_then(|a| {
            a.elements
                .iter()
                .find(|e| e.key.eq_ignore_ascii_case("name"))
        })
        .map(|e| e.value.clone())
        .unwrap_or_else(|| "SiddhiApp".to_string());

    let mut app = SiddhiApp::new(app_name);
    for ann in &annotations {
        app.add_annotation(ann.clone());
    }

    let mut parts = s_no_ann.split(';').peekable();
    while let Some(part) = parts.next() {
        let part_trimmed = part.trim();
        if part_trimmed.is_empty() {
            continue;
        }
        let mut stmt = part_trimmed.to_string();
        let mut lower = stmt.to_lowercase();

        if lower.starts_with("partition") {
            while !lower.trim_end().ends_with("end") {
                if let Some(next_part) = parts.next() {
                    stmt.push(';');
                    stmt.push_str(next_part);
                    lower = stmt.to_lowercase();
                } else {
                    break;
                }
            }
            let p = parse_partition(&stmt)?;
            app.add_execution_element(ExecutionElement::Partition(p));
        } else if lower.contains("define stream") {
            let def = parse_stream_definition(&stmt)?;
            app.add_stream_definition(def);
        } else if lower.contains("define table") {
            let def = parse_table_definition(&stmt)?;
            app.add_table_definition(def);
        } else if lower.contains("define window") {
            let def = parse_window_definition(&stmt)?;
            app.add_window_definition(def);
        } else if lower.contains("define function") {
            let def = parse_function_definition(&stmt)?;
            app.add_function_definition(def);
        } else if lower.contains("define trigger") {
            let def = parse_trigger_definition(&stmt)?;
            app.add_trigger_definition(def);
        } else if lower.contains("define aggregation") {
            let def = parse_aggregation_definition(&stmt)?;
            app.add_aggregation_definition(def);
        } else if lower.starts_with("from") {
            let q = parse_query(&stmt)?;
            app.add_execution_element(ExecutionElement::Query(q));
        }
    }

    Ok(app)
}

pub fn parse_stream_definition(stream_def_string: &str) -> Result<StreamDefinition, String> {
    let preprocessed = preprocess_siddhi_app(stream_def_string);
    let s = update_variables(&preprocessed)?;
    grammar::StreamDefParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

pub fn parse_table_definition(table_def_string: &str) -> Result<TableDefinition, String> {
    let preprocessed = preprocess_siddhi_app(table_def_string);
    let s = update_variables(&preprocessed)?;
    grammar::TableDefParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

pub fn parse_window_definition(window_def_string: &str) -> Result<WindowDefinition, String> {
    let preprocessed = preprocess_siddhi_app(window_def_string);
    let s = update_variables(&preprocessed)?;
    grammar::WindowDefParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

pub fn parse_aggregation_definition(agg_def_string: &str) -> Result<AggregationDefinition, String> {
    let preprocessed = preprocess_siddhi_app(agg_def_string);
    let s = update_variables(&preprocessed)?;
    grammar::AggDefParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

pub fn parse_partition(partition_string: &str) -> Result<Partition, String> {
    let preprocessed = preprocess_siddhi_app(partition_string);
    let s = update_variables(&preprocessed)?;
    grammar::PartitionStmtParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

pub fn parse_query(query_string: &str) -> Result<Query, String> {
    let preprocessed = preprocess_siddhi_app(query_string);
    let s = update_variables(&preprocessed)?;
    grammar::QueryStmtParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

pub fn parse_function_definition(func_def_string: &str) -> Result<FunctionDefinition, String> {
    let preprocessed = preprocess_siddhi_app(func_def_string);
    let s = update_variables(&preprocessed)?;
    grammar::FunctionDefParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

pub fn parse_trigger_definition(trig_def_string: &str) -> Result<TriggerDefinition, String> {
    let preprocessed = preprocess_siddhi_app(trig_def_string);
    let s = update_variables(&preprocessed)?;
    grammar::TriggerDefParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

// Renamed from parseTimeConstantDefinition to align with type name
// Java method returns io.siddhi.query.api.expression.constant.TimeConstant
// Our ExpressionConstant is crate::query_api::expression::constant::Constant
pub fn parse_time_constant(time_const_string: &str) -> Result<ExpressionConstant, String> {
    let preprocessed = preprocess_siddhi_app(time_const_string);
    let s = update_variables(&preprocessed)?;
    grammar::TimeConstantParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

pub fn parse_on_demand_query(query_string: &str) -> Result<OnDemandQuery, String> {
    let preprocessed = preprocess_siddhi_app(query_string);
    let s = update_variables(&preprocessed)?;
    grammar::OnDemandQueryStmtParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

// parseStoreQuery in Java calls parseOnDemandQuery.
pub fn parse_store_query(store_query_string: &str) -> Result<StoreQuery, String> {
    let preprocessed = preprocess_siddhi_app(store_query_string);
    let s = update_variables(&preprocessed)?;
    grammar::StoreQueryStmtParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

pub fn parse_expression(expr_string: &str) -> Result<Expression, String> {
    let preprocessed = preprocess_siddhi_app(expr_string);
    let s = update_variables(&preprocessed)?;
    grammar::ExpressionParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

pub fn parse_set_clause(set_clause_string: &str) -> Result<UpdateSet, String> {
    let preprocessed = preprocess_siddhi_app(set_clause_string);
    let s = update_variables(&preprocessed)?;
    grammar::SetClauseParser::new()
        .parse(&s)
        .map_err(|e| format_parse_error(e, &s))
}

// The problematic comments and `pub use` below are removed.
// `update_variables` is now directly public.

/// Formats LALRPOP parse errors with better context and line information
fn format_parse_error<T: std::fmt::Debug>(err: T, source: &str) -> String {
    let err_str = format!("{:?}", err);
    
    // Try to extract position information from error
    if let Some(pos_start) = err_str.find("token: (") {
        if let Some(pos_info) = err_str.get(pos_start + 9..) {
            if let Some(pos_end) = pos_info.find(',') {
                if let Ok(position) = pos_info[..pos_end].parse::<usize>() {
                    let (line, col) = calculate_line_position(source, position);
                    let line_content = source.lines().nth(line - 1).unwrap_or("");
                    return format!(
                        "Parse error at line {}, column {}:\n  {}\n  {}^\nDetails: {}",
                        line, col, line_content,
                        " ".repeat(col - 1),
                        err_str
                    );
                }
            }
        }
    }
    
    err_str
}
