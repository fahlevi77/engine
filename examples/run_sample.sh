#!/bin/sh
# Run a Siddhi app with the CLI. Pass the path to a .siddhi file or run the
# basic sample if no argument is provided.

APP_PATH=${1:-examples/sample.siddhi}
cargo run --bin run_siddhi "$APP_PATH"
