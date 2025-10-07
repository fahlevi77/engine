#!/bin/sh
# Run a EventFlux app with the CLI. Pass the path to a .eventflux file or run the
# basic sample if no argument is provided.

APP_PATH=${1:-examples/sample.eventflux}
cargo run --bin run_eventflux "$APP_PATH"
