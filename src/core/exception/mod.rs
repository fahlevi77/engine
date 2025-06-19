pub mod error;
pub use error::SiddhiError;
// ErrorStore types now live under the stream::output module
pub use crate::core::stream::output::error_store::{ErrorStore, InMemoryErrorStore};
