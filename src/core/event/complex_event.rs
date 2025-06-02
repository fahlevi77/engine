// siddhi_rust/src/core/event/complex_event.rs
// Corresponds to io.siddhi.core.event.ComplexEvent (interface)
use super::value::AttributeValue;
use std::fmt::Debug;

// This enum corresponds to ComplexEvent.Type in Java
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ComplexEventType {
    #[default] Current, // Mapped from CURRENT
    Expired,
    Timer,
    Reset,
}

// Using a trait for ComplexEvent as it's an interface in Java.
// Concrete event types like StreamEvent and StateEvent will implement this.
pub trait ComplexEvent: Debug + Send + Sync + 'static { // 'static if Box<dyn ComplexEvent> is stored long-term
    // Linked list structure
    fn get_next(&self) -> Option<&dyn ComplexEvent>;
    // set_next might need to take Box<dyn ComplexEvent> or a concrete type if all links are same.
    // For flexibility with different implementations, Box<dyn ComplexEvent> is better.
    fn set_next(&mut self, next_event: Option<Box<dyn ComplexEvent>>);

    // Data access
    // Java's getOutputData() returns Object[]. If output_data is not always present, Option is good.
    fn get_output_data(&self) -> Option<&[AttributeValue]>; // Slice for read-only access
    // Java's setOutputData(Object object, int index) implies output_data is an array.
    // A more Rusty way might be set_output_data_vec(Vec<AttributeValue>) or similar.
    // For direct porting of setOutputData(Object, int):
    fn set_output_data_at_idx(&mut self, value: AttributeValue, index: usize) -> Result<(), String>;
    // Or a method to set the whole output data if it's optional:
    fn set_output_data_vec(&mut self, data: Option<Vec<AttributeValue>>);


    fn get_timestamp(&self) -> i64;
    fn set_timestamp(&mut self, timestamp: i64); // Added setter for consistency, though not in Java interface

    // getAttribute(int[] position) and setAttribute(Object object, int[] position)
    // These are complex due to the int[] position encoding type and index.
    // This might be better handled by specific methods on StreamEvent and StateEvent
    // if the position encoding is specific to them.
    // For a generic trait, this is hard to define well without knowing position structure.
    // fn get_attribute_by_position(&self, position: &[i32]) -> Option<AttributeValue>;
    // fn set_attribute_by_position(&mut self, value: AttributeValue, position: &[i32]) -> Result<(), String>;

    fn get_event_type(&self) -> ComplexEventType; // Renamed from getType
    fn set_event_type(&mut self, event_type: ComplexEventType); // Renamed from setType

    // Helper for downcasting if needed, common pattern for trait objects.
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    // Potentially a clone_box method for dynamic cloning
    // fn clone_box(&self) -> Box<dyn ComplexEvent>;
}

// Example of how a concrete type might implement clone_box
// impl Clone for Box<dyn ComplexEvent> {
//     fn clone(&self) -> Self {
//         self.clone_box()
//     }
// }
