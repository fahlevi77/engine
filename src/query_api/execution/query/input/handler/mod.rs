// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to package io.eventflux.query.api.execution.query.input.handler

pub mod filter;
pub mod stream_function;
pub mod stream_handler; // Interface, will be a trait or enum
pub mod window; // Renamed to window_handler internally if needed to avoid conflict

// Re-export key types
pub use self::filter::Filter;
pub use self::stream_function::StreamFunction;
pub use self::stream_handler::StreamHandler; // This will be the main enum/trait
pub use self::window::Window as WindowHandler; // Alias to avoid conflicts if necessary elsewhere

// The StreamHandler will likely be an enum in Rust:
// pub enum StreamHandler {
//     Filter(Filter),
//     Function(StreamFunction),
//     Window(WindowHandler),
// }
// Each struct (Filter, StreamFunction, WindowHandler) will implement EventFluxElement
// and potentially a common trait if methods beyond EventFluxElement are shared from Java's StreamHandler.
// Java's StreamHandler has `getParameters()` and EventFluxElement context.
// The Extension interface is implemented by StreamFunction and Window.
