use siddhi_rust::query_compiler::{parse_stream_definition, parse_table_definition, parse_window_definition, parse_query};
use siddhi_rust::query_api::definition::attribute::Type as AttrType;
use siddhi_rust::query_api::execution::query::input::stream::input_stream::InputStreamTrait;
use siddhi_rust::query_api::execution::query::input::{InputStream, JoinType, StateInputStreamType};

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

#[test]
fn test_parse_join_query() {
    let q = parse_query("from Left join Right on cond select a, b insert into Out").unwrap();
    match q.get_input_stream().unwrap() {
        InputStream::Join(j) => {
            assert_eq!(j.left_input_stream.get_stream_id_str(), "Left");
            assert_eq!(j.right_input_stream.get_stream_id_str(), "Right");
            assert_eq!(j.join_type, JoinType::Join);
        }
        _ => panic!("expected join"),
    }
    assert_eq!(q.get_selector().selection_list.len(), 2);
}

#[test]
fn test_parse_left_outer_join_query() {
    let q = parse_query("from L left outer join R on cond select x, y insert into O").unwrap();
    match q.get_input_stream().unwrap() {
        InputStream::Join(j) => {
            assert_eq!(j.join_type, JoinType::LeftOuterJoin);
        }
        _ => panic!("expected join"),
    }
}

#[test]
fn test_parse_pattern_query() {
    let q = parse_query("from every e1=Stream1 -> e2=Stream2 select e1,e2 insert into Out").unwrap();
    match q.get_input_stream().unwrap() {
        InputStream::State(st) => {
            assert_eq!(st.state_type, StateInputStreamType::Pattern);
            assert_eq!(st.get_all_stream_ids(), vec!["Stream1", "Stream2"]);
        }
        _ => panic!("expected pattern state"),
    }
}

#[test]
fn test_parse_sequence_query() {
    let q = parse_query("from every s1=StreamA, s2=StreamB select s1,s2 insert into O").unwrap();
    match q.get_input_stream().unwrap() {
        InputStream::State(st) => {
            assert_eq!(st.state_type, StateInputStreamType::Sequence);
            assert_eq!(st.get_all_stream_ids(), vec!["StreamA", "StreamB"]);
        }
        _ => panic!("expected sequence state"),
    }
}
