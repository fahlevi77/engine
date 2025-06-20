#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;

#[test]
fn inner_join_simple() {
    let app = "\
        define stream L (id int);\n\
        define stream R (id int);\n\
        define stream Out (l int, r int);\n\
        from L join R on L.id == R.id select L.id as l, R.id as r insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("L", vec![AttributeValue::Int(1)]);
    runner.send("R", vec![AttributeValue::Int(1)]);
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![vec![AttributeValue::Int(1), AttributeValue::Int(1)]]
    );
}

#[test]
fn left_outer_join_no_match() {
    let app = "\
        define stream L (id int);\n\
        define stream R (id int);\n\
        define stream Out (l int, r int);\n\
        from L left outer join R on L.id == R.id select L.id as l, R.id as r insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("L", vec![AttributeValue::Int(2)]);
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![vec![AttributeValue::Int(2), AttributeValue::Null]]
    );
}

#[test]
fn join_with_condition_gt() {
    let app = "\
        define stream L (id int);\n\
        define stream R (id int);\n\
        define stream Out (l int, r int);\n\
        from L join R on L.id > R.id select L.id as l, R.id as r insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("L", vec![AttributeValue::Int(1)]);
    runner.send("L", vec![AttributeValue::Int(3)]);
    runner.send("R", vec![AttributeValue::Int(1)]);
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![vec![AttributeValue::Int(3), AttributeValue::Int(1)]]
    );
}

#[test]
fn join_complex_condition() {
    let app = "\
        define stream L (id int);\n\
        define stream R (id int);\n\
        define stream Out (l int, r int);\n\
        from L join R on (L.id > R.id and R.id > 0) or L.id == 10 select L.id as l, R.id as r insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("L", vec![AttributeValue::Int(1)]);
    runner.send("R", vec![AttributeValue::Int(1)]);
    runner.send("L", vec![AttributeValue::Int(10)]);
    runner.send("R", vec![AttributeValue::Int(2)]);
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![
            vec![AttributeValue::Int(10), AttributeValue::Int(1)],
            vec![AttributeValue::Int(10), AttributeValue::Int(2)],
        ]
    );
}

#[test]
fn right_outer_join_no_match() {
    let app = "\
        define stream L (id int);\n\
        define stream R (id int);\n\
        define stream Out (l int, r int);\n\
        from L right outer join R on L.id == R.id select L.id as l, R.id as r insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("R", vec![AttributeValue::Int(5)]);
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Null, AttributeValue::Int(5)]]);
}

#[test]
fn full_outer_join_basic() {
    let app = "\
        define stream L (id int);\n\
        define stream R (id int);\n\
        define stream Out (l int, r int);\n\
        from L full outer join R on L.id == R.id select L.id as l, R.id as r insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("L", vec![AttributeValue::Int(1)]);
    runner.send("R", vec![AttributeValue::Int(2)]);
    let out = runner.shutdown();
    assert!(out.contains(&vec![AttributeValue::Int(1), AttributeValue::Null]));
    assert!(out.contains(&vec![AttributeValue::Null, AttributeValue::Int(2)]));
}

#[test]
fn join_with_group_by() {
    let app = "\
        define stream L (id int, cat int);\n\
        define stream R (id int);\n\
        define stream Out (cat int, c long);\n\
        from L join R on L.id == R.id\n\
        select L.cat as cat, count() as c\n\
        group by cat order by cat asc insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send_batch(
        "L",
        vec![
            vec![AttributeValue::Int(1), AttributeValue::Int(10)],
            vec![AttributeValue::Int(1), AttributeValue::Int(10)],
            vec![AttributeValue::Int(2), AttributeValue::Int(20)],
        ],
    );
    runner.send_batch(
        "R",
        vec![
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(2)],
            vec![AttributeValue::Int(1)],
        ],
    );
    let out = runner.shutdown();
    let expected = vec![
        vec![AttributeValue::Int(10), AttributeValue::Long(1)],
        vec![AttributeValue::Int(10), AttributeValue::Long(2)],
        vec![AttributeValue::Int(20), AttributeValue::Long(1)],
        vec![AttributeValue::Int(10), AttributeValue::Long(3)],
        vec![AttributeValue::Int(10), AttributeValue::Long(4)],
    ];
    assert_eq!(out, expected);
}
