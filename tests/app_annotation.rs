use siddhi_rust::query_compiler::parse;

#[test]
fn test_parse_app_annotations() {
    let input = "@app:name('Foo')\n@app:description('Bar')\n\ndefine stream S (val int);";
    let app = parse(input).expect("parse");
    assert_eq!(app.name, "Foo");
    assert_eq!(app.annotations.len(), 2);
    assert!(app.annotations.iter().any(|a| a.name == "app" && a.elements.iter().any(|e| e.key == "description" && e.value == "Bar")));
}
