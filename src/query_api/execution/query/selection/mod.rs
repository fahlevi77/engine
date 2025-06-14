pub mod order_by_attribute;
pub mod output_attribute;
pub mod selector;

pub use self::order_by_attribute::{Order, OrderByAttribute}; // Order enum from order_by_attribute
pub use self::output_attribute::OutputAttribute;
pub use self::selector::Selector;

// BasicSelector.java is not translated as a separate struct.
// Its functionality (builder methods returning Self) is part of Selector.java,
// and thus part of the Rust Selector struct. If specific dispatch or typing for
// BasicSelector is needed later, it could be added as a wrapper or distinct type.
