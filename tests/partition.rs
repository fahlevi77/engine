use siddhi_rust::query_api::execution::ExecutionElement;
use siddhi_rust::query_compiler::{parse, parse_partition};

#[test]
fn test_parse_partition_direct() {
    let part = "partition with (symbol of InStream) begin from InStream select sum(volume) as sumvolume insert into OutStream; end";
    let p = parse_partition(part).expect("parse partition");
    assert_eq!(p.query_list.len(), 1);
    assert_eq!(p.partition_type_map.len(), 1);
}

#[test]
fn test_parse_app_with_partition() {
    let app = "\
        define stream InStream (symbol string, volume int);\n\
        define stream OutStream (sumvolume long);\n\
        partition with (symbol of InStream) begin from InStream select sum(volume) as sumvolume insert into OutStream; end;\n";
    let sa = parse(app).expect("parse");
    assert_eq!(sa.get_execution_elements().len(), 1);
    match &sa.get_execution_elements()[0] {
        ExecutionElement::Partition(p) => {
            assert_eq!(p.query_list.len(), 1);
        }
        _ => panic!("expected partition"),
    }
}
