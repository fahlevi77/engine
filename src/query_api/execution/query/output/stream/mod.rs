// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod set_attribute;
pub mod update_set; // Added this line

pub use self::set_attribute::SetAttribute;
pub use self::update_set::UpdateSet; // Added this line

// This module will also contain other output stream action structs:
// InsertIntoStreamAction, ReturnStreamAction, DeleteStreamAction, etc.
// Currently, these are defined directly in `output_stream.rs` (the file that defines the OutputStream enum).
// For better organization, those action structs (InsertIntoStreamAction, etc.) could be moved into this `stream` module
// as well, each in their own file (e.g., insert_into_stream_action.rs).
// And then `output_stream.rs` would `use super::stream::actions::*;` or similar.
// For now, keeping action structs in `output_stream.rs` as per current structure.
