use siddhi_rust::query_compiler::{parse_stream_definition, parse_table_definition, parse_window_definition, parse_query};
use siddhi_rust::query_api::definition::attribute::Type as AttrType;
use siddhi_rust::query_api::execution::query::input::stream::input_stream::InputStreamTrait;

#[test]
fn test_parse_stream_definition() {
    let def = parse_stream_definition("define stream InputStream (name string, age int)").unwrap();
    assert_eq!(def.abstract_definition.id, "InputStream");
    let attrs = &def.abstract_definition.attribute_list;
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].name, "name");
    assert_eq!(attrs[0].attribute_type, AttrType::STRING);
    assert_eq!(attrs[1].name, "age");
    assert_eq!(attrs[1].attribute_type, AttrType::INT);
}

#[test]
fn test_parse_table_definition() {
    let def = parse_table_definition("define table MyTable (symbol string, price double)").unwrap();
    assert_eq!(def.abstract_definition.id, "MyTable");
    let attrs = &def.abstract_definition.attribute_list;
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].name, "symbol");
    assert_eq!(attrs[0].attribute_type, AttrType::STRING);
    assert_eq!(attrs[1].attribute_type, AttrType::DOUBLE);
}

#[test]
fn test_parse_query() {
    let q = parse_query("from Input select name, age insert into Out").unwrap();
    assert_eq!(q.get_input_stream().unwrap().get_all_stream_ids()[0], "Input");
    assert_eq!(q.get_output_stream().get_target_id().unwrap(), "Out");
    assert_eq!(q.get_selector().selection_list.len(), 2);
}

#[test]
fn test_parse_window_definition() {
    let def = parse_window_definition("define window Win (symbol string) length(5)").unwrap();
    assert_eq!(def.stream_definition.abstract_definition.id, "Win");
    let attrs = &def.stream_definition.abstract_definition.attribute_list;
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].name, "symbol");
    assert_eq!(attrs[0].attribute_type, AttrType::STRING);
    let handler = def.window_handler.expect("window handler");
    assert_eq!(handler.name, "length");
    assert_eq!(handler.parameters.len(), 1);
}
