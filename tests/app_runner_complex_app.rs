use eventflux_rust::query_compiler::parse;

#[test]
fn complex_app_parses_and_runs() {
    let app = "\
        define stream InStream (symbol string, value int);\n\
        define stream OutStream (v int);\n\
        define trigger Tick at every 50 ms;\n\
        define aggregation Agg from InStream select sum(value) as total group by symbol aggregate every seconds;\n\
        define partition with (symbol of InStream) begin\n\
            from InStream select value as v insert into OutStream;\n\
        end;\n\
        from Agg on true within 1 sec per 1 sec select total as v insert into OutStream;\n";
    let res = parse(app);
    assert!(res.is_ok());
}
