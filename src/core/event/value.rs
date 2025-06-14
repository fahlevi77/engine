// siddhi_rust/src/core/event/value.rs
use std::any::Any;
use std::fmt;

// This enum will represent the possible types of attribute values in a Siddhi event.
// Java Event uses Object[], allowing any type. Rust needs to be more explicit.
// query_api::definition::attribute::Type enum has STRING, INT, LONG, FLOAT, DOUBLE, BOOL, OBJECT.
// This enum should reflect those types for data carrying.

pub enum AttributeValue {
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Object(Option<Box<dyn Any + Send + Sync>>), // For OBJECT type, ensure thread safety
    Null,                                       // To represent null values explicitly
}

// Manual implementation of Debug to handle Box<dyn Any>
impl fmt::Debug for AttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributeValue::String(s) => write!(f, "String({:?})", s),
            AttributeValue::Int(i) => write!(f, "Int({:?})", i),
            AttributeValue::Long(l) => write!(f, "Long({:?})", l),
            AttributeValue::Float(fl) => write!(f, "Float({:?})", fl),
            AttributeValue::Double(d) => write!(f, "Double({:?})", d),
            AttributeValue::Bool(b) => write!(f, "Bool({:?})", b),
            AttributeValue::Object(_) => write!(f, "Object(<opaque>)"), // Cannot inspect Box<dyn Any> easily
            AttributeValue::Null => write!(f, "Null"),
        }
    }
}

// Manual implementation of PartialEq
impl PartialEq for AttributeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AttributeValue::String(a), AttributeValue::String(b)) => a == b,
            (AttributeValue::Int(a), AttributeValue::Int(b)) => a == b,
            (AttributeValue::Long(a), AttributeValue::Long(b)) => a == b,
            (AttributeValue::Float(a), AttributeValue::Float(b)) => a == b, // Note: float comparison issues
            (AttributeValue::Double(a), AttributeValue::Double(b)) => a == b, // Note: float comparison issues
            (AttributeValue::Bool(a), AttributeValue::Bool(b)) => a == b,
            (AttributeValue::Null, AttributeValue::Null) => true,
            // Comparing Box<dyn Any> is problematic.
            // Typically, objects are compared by reference or specific methods, not direct equality.
            // For now, objects are not equal unless they are the same instance (which this comparison doesn't check)
            // or if we had a way to downcast and compare known types.
            (AttributeValue::Object(_), AttributeValue::Object(_)) => false,
            _ => false, // Different enum variants
        }
    }
}

impl Clone for AttributeValue {
    fn clone(&self) -> Self {
        match self {
            AttributeValue::String(s) => AttributeValue::String(s.clone()),
            AttributeValue::Int(i) => AttributeValue::Int(*i),
            AttributeValue::Long(l) => AttributeValue::Long(*l),
            AttributeValue::Float(f) => AttributeValue::Float(*f),
            AttributeValue::Double(d) => AttributeValue::Double(*d),
            AttributeValue::Bool(b) => AttributeValue::Bool(*b),
            AttributeValue::Object(_) => AttributeValue::Object(None),
            AttributeValue::Null => AttributeValue::Null,
        }
    }
}

impl Default for AttributeValue {
    fn default() -> Self {
        AttributeValue::Null
    }
}

// Potential helper methods for AttributeValue:
impl AttributeValue {
    // pub fn as_string(&self) -> Option<&String> { ... }
    // pub fn as_i32(&self) -> Option<i32> { ... }
    // ... and so on for other types
}

impl fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributeValue::String(s) => write!(f, "{}", s),
            AttributeValue::Int(i) => write!(f, "{}", i),
            AttributeValue::Long(l) => write!(f, "{}", l),
            AttributeValue::Float(v) => write!(f, "{}", v),
            AttributeValue::Double(v) => write!(f, "{}", v),
            AttributeValue::Bool(b) => write!(f, "{}", b),
            AttributeValue::Object(_) => write!(f, "<object>"),
            AttributeValue::Null => write!(f, "null"),
        }
    }
}
