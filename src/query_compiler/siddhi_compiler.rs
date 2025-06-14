// Note: query_api types are used for return types.
// Ensure these paths are correct based on query_api module structure and re-exports.
use crate::query_api::{
    definition::{
        attribute::Type as AttributeType,
        AggregationDefinition,
        FunctionDefinition,
        StreamDefinition,
        TableDefinition,
        TriggerDefinition,
        WindowDefinition,
    },
    execution::{
        query::{
            input::InputStream,
            output::output_stream::{InsertIntoStreamAction, OutputStream, OutputStreamAction},
            selection::Selector,
            OnDemandQuery, Query, StoreQuery,
        },
        ExecutionElement, Partition,
    },
    expression::{constant::Constant as ExpressionConstant, variable::Variable, Expression},
    SiddhiApp,
};
use lalrpop_util::lalrpop_mod;
use regex::Regex;
use std::env;

lalrpop_mod!(pub grammar, "/query_compiler/grammar.rs");

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

    let re = Regex::new(r"\$\{(\w+)\}").map_err(|e| e.to_string())?;
    let mut updated_siddhi_app = String::new();
    let mut last_match_end = 0;

    for captures in re.captures_iter(siddhi_app_string) {
        let full_match = captures.get(0).unwrap(); // The whole ${varName}
        let var_name = captures.get(1).unwrap().as_str();

        updated_siddhi_app.push_str(&siddhi_app_string[last_match_end..full_match.start()]);

        match env::var(var_name) {
            Ok(value) => updated_siddhi_app.push_str(&value),
            Err(_) => {
                // The Java code throws SiddhiParserException with context.
                // For now, returning a simpler error.
                // TODO: Enhance error reporting with line numbers if possible without full parser.
                return Err(format!(
                    "No system or environmental variable found for '${{{}}}'",
                    var_name
                ));
            }
        }
        last_match_end = full_match.end();
    }
    updated_siddhi_app.push_str(&siddhi_app_string[last_match_end..]);
    Ok(updated_siddhi_app)
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
        _ => Err(format!("Unknown attribute type: {}", t)),
    }
}

pub fn parse(siddhi_app_string: &str) -> Result<SiddhiApp, String> {
    let s = update_variables(siddhi_app_string)?;

    let mut annotations = Vec::new();
    let mut lines_without_ann = Vec::new();
    for line in s.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('@') {
            if let Ok(ann) = grammar::AnnotationStmtParser::new().parse(trimmed) {
                annotations.push(ann);
                continue;
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
        let mut stmt = part.trim().to_string();
        if stmt.is_empty() {
            continue;
        }
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
        } else if lower.starts_with("define stream") {
            let def = parse_stream_definition(&stmt)?;
            app.add_stream_definition(def);
        } else if lower.starts_with("define table") {
            let def = parse_table_definition(&stmt)?;
            app.add_table_definition(def);
        } else if lower.starts_with("define window") {
            let def = parse_window_definition(&stmt)?;
            app.add_window_definition(def);
        } else if lower.starts_with("define function") {
            let def = parse_function_definition(&stmt)?;
            app.add_function_definition(def);
        } else if lower.starts_with("define trigger") {
            let def = parse_trigger_definition(&stmt)?;
            app.add_trigger_definition(def);
        } else if lower.starts_with("define aggregation") {
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
    let s = update_variables(stream_def_string)?;
    grammar::StreamDefParser::new()
        .parse(&s)
        .map_err(|e| format!("{:?}", e))
}

pub fn parse_table_definition(table_def_string: &str) -> Result<TableDefinition, String> {
    let s = update_variables(table_def_string)?;
    grammar::TableDefParser::new()
        .parse(&s)
        .map_err(|e| format!("{:?}", e))
}

pub fn parse_window_definition(window_def_string: &str) -> Result<WindowDefinition, String> {
    let s = update_variables(window_def_string)?;
    grammar::WindowDefParser::new()
        .parse(&s)
        .map_err(|e| format!("{:?}", e))
}

pub fn parse_aggregation_definition(agg_def_string: &str) -> Result<AggregationDefinition, String> {
    let s = update_variables(agg_def_string)?;
    grammar::AggDefParser::new()
        .parse(&s)
        .map_err(|e| format!("{:?}", e))
}

pub fn parse_partition(partition_string: &str) -> Result<Partition, String> {
    let s = update_variables(partition_string)?;
    grammar::PartitionStmtParser::new()
        .parse(&s)
        .map_err(|e| format!("{:?}", e))
}

pub fn parse_query(query_string: &str) -> Result<Query, String> {
    let s = update_variables(query_string)?;
    grammar::QueryStmtParser::new()
        .parse(&s)
        .map_err(|e| format!("{:?}", e))
}

pub fn parse_function_definition(func_def_string: &str) -> Result<FunctionDefinition, String> {
    let s = update_variables(func_def_string)?;
    grammar::FunctionDefParser::new()
        .parse(&s)
        .map_err(|e| format!("{:?}", e))
}

pub fn parse_trigger_definition(trig_def_string: &str) -> Result<TriggerDefinition, String> {
    let s = update_variables(trig_def_string)?;
    grammar::TriggerDefParser::new()
        .parse(&s)
        .map_err(|e| format!("{:?}", e))
}

// Renamed from parseTimeConstantDefinition to align with type name
// Java method returns io.siddhi.query.api.expression.constant.TimeConstant
// Our ExpressionConstant is crate::query_api::expression::constant::Constant
pub fn parse_time_constant(time_const_string: &str) -> Result<ExpressionConstant, String> {
    let s = update_variables(time_const_string)?;
    grammar::TimeConstantParser::new()
        .parse(&s)
        .map_err(|e| format!("{:?}", e))
}

pub fn parse_on_demand_query(query_string: &str) -> Result<OnDemandQuery, String> {
    let s = update_variables(query_string)?;
    grammar::OnDemandQueryStmtParser::new()
        .parse(&s)
        .map_err(|e| format!("{:?}", e))
}

// parseStoreQuery in Java calls parseOnDemandQuery.
pub fn parse_store_query(store_query_string: &str) -> Result<StoreQuery, String> {
    let s = update_variables(store_query_string)?;
    grammar::StoreQueryStmtParser::new()
        .parse(&s)
        .map_err(|e| format!("{:?}", e))
}

pub fn parse_expression(expr_string: &str) -> Result<Expression, String> {
    let s = update_variables(expr_string)?;
    grammar::ExpressionParser::new()
        .parse(&s)
        .map_err(|e| format!("{:?}", e))
}

// The problematic comments and `pub use` below are removed.
// `update_variables` is now directly public.
