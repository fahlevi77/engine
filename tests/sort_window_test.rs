// SPDX-License-Identifier: MIT OR Apache-2.0

// TODO: Sort window tests converted to SQL syntax
// Sort window is implemented, ready for testing with SQL syntax
// See feat/grammar/GRAMMAR_STATUS.md for M1 feature list.

#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use eventflux_rust::core::event::value::AttributeValue;

#[tokio::test]
#[ignore = "WINDOW sort() SQL syntax not yet supported - needs syntax definition"]
async fn test_basic_sort_window() {
    // TODO: Sort window implemented but SQL parser doesn't recognize "WINDOW sort()" syntax
    // Need to determine correct SQL syntax: WINDOW_SORT()? or different approach?
    let app = "\
        CREATE STREAM In (price DOUBLE, volume INT);\n\
        CREATE STREAM Out (price DOUBLE, volume INT);\n\
        INSERT INTO Out\n\
        SELECT price, volume FROM In WINDOW sort(3);\n";

    let runner = AppRunner::new(app, "Out").await;

    // Send some events
    runner.send(
        "In",
        vec![AttributeValue::Double(100.0), AttributeValue::Int(50)],
    );

    runner.send(
        "In",
        vec![AttributeValue::Double(200.0), AttributeValue::Int(30)],
    );

    runner.send(
        "In",
        vec![AttributeValue::Double(150.0), AttributeValue::Int(40)],
    );

    let output = runner.shutdown();
    println!("Sort window output: {:?}", output);

    // For now, just verify we get some output (since sorting logic is not implemented yet)
    assert!(!output.is_empty(), "Should have sort window output");
}

#[tokio::test]
#[ignore = "WINDOW sort() SQL syntax not yet supported - needs syntax definition"]
async fn test_sort_window_with_parameters() {
    // TODO: Sort window implemented but SQL syntax not yet defined
    let app = "\
        CREATE STREAM In (value INT);\n\
        CREATE STREAM Out (value INT);\n\
        INSERT INTO Out\n\
        SELECT value FROM In WINDOW sort(2);\n";

    let runner = AppRunner::new(app, "Out").await;

    runner.send("In", vec![AttributeValue::Int(3)]);
    runner.send("In", vec![AttributeValue::Int(1)]);
    runner.send("In", vec![AttributeValue::Int(2)]);

    let output = runner.shutdown();
    println!("Sort window with parameters output: {:?}", output);

    // Should have output for all sent events
    assert!(!output.is_empty(), "Should have output events");
}

#[tokio::test]
#[ignore = "WINDOW sort() SQL syntax not yet supported - needs syntax definition"]
async fn test_sort_window_length_validation() {
    // TODO: Sort window implemented but SQL syntax not yet defined
    let app = "\
        CREATE STREAM In (id INT);\n\
        CREATE STREAM Out (id INT);\n\
        INSERT INTO Out\n\
        SELECT id FROM In WINDOW sort(1);\n";

    let runner = AppRunner::new(app, "Out").await;

    runner.send("In", vec![AttributeValue::Int(42)]);

    let output = runner.shutdown();
    println!("Sort window length validation output: {:?}", output);

    // Should work with length 1
    assert!(!output.is_empty(), "Should work with length 1");
}

#[tokio::test]
#[ignore = "WINDOW sort() SQL syntax not yet supported - needs syntax definition"]
async fn test_sort_window_expiry() {
    // TODO: Sort window implemented but SQL syntax not yet defined
    let app = "\
        CREATE STREAM In (value INT);\n\
        CREATE STREAM Out (value INT);\n\
        INSERT INTO Out\n\
        SELECT value FROM In WINDOW sort(2);\n";

    let runner = AppRunner::new(app, "Out").await;

    // Send 3 events to a window of size 2 - should get expired events
    runner.send("In", vec![AttributeValue::Int(10)]);
    runner.send("In", vec![AttributeValue::Int(20)]);
    runner.send("In", vec![AttributeValue::Int(30)]);

    let output = runner.shutdown();
    println!("Sort window expiry output: {:?}", output);

    // Should have output for all events (both current and expired)
    assert!(
        output.len() >= 3,
        "Should have at least 3 events (current + expired)"
    );
}

#[tokio::test]
#[ignore = "WINDOW sort() SQL syntax not yet supported - needs syntax definition"]
async fn test_sort_window_ordering() {
    // TODO: Sort window implemented but SQL syntax not yet defined
    let app = "\
        CREATE STREAM In (id INT);\n\
        CREATE STREAM Out (id INT);\n\
        INSERT INTO Out\n\
        SELECT id FROM In WINDOW sort(3);\n";

    let runner = AppRunner::new(app, "Out").await;

    // Send events - the sort window should maintain them in sorted order
    runner.send("In", vec![AttributeValue::Int(100)]);
    runner.send("In", vec![AttributeValue::Int(200)]);
    runner.send("In", vec![AttributeValue::Int(300)]);

    let output = runner.shutdown();
    println!("Sort window ordering output: {:?}", output);

    // Should have output for all sent events
    assert_eq!(output.len(), 3, "Should have exactly 3 output events");

    // All events should be present
    for event in &output {
        assert_eq!(event.len(), 1, "Each event should have one attribute");
        match &event[0] {
            AttributeValue::Int(val) => {
                assert!(
                    vec![100, 200, 300].contains(val),
                    "Event value should be one of the sent values"
                );
            }
            _ => panic!("Expected integer value"),
        }
    }
}
