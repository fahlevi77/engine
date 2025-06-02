pub mod stream_junction;
pub mod input;
pub mod output;

pub use self::stream_junction::{StreamJunction, OnErrorAction, Receiver as StreamJunctionReceiver}; // Added Receiver
pub use self::input::InputHandler; // Re-exporting key types from submodules
pub use self::input::InputManager;
// pub use self::input::InputProcessor; // Trait, might be re-exported if used externally
pub use self::output::StreamCallback;
