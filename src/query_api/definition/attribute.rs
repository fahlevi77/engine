// Corresponds to io.siddhi.query.api.definition.Attribute
use crate::query_api::siddhi_element::SiddhiElement;

/// Defines the data type of an attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] // Added Copy for easier usage
pub enum Type {
    STRING,
    INT,
    LONG,
    FLOAT,
    DOUBLE,
    BOOL,
    OBJECT,
}

impl Default for Type {
    fn default() -> Self {
        Type::OBJECT
    } // Default type as per From<SiddhiElement> impl
}

/// Represents an attribute with a name and a type.
#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct Attribute {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // Attribute fields
    pub name: String,
    pub attribute_type: Type,
}

impl Attribute {
    pub fn new(name: String, attribute_type: Type) -> Self {
        Attribute {
            siddhi_element: SiddhiElement::default(),
            name,
            attribute_type,
        }
    }

    // Getter methods
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_type(&self) -> &Type {
        // Changed to return &Type to avoid clone if Type is not Copy
        &self.attribute_type
    }
}

// The From<SiddhiElement> for Attribute impl is removed as it's less relevant
// when Attribute composes SiddhiElement directly. Construction is via new().

// If direct access to SiddhiElement fields from Attribute is desired:
// impl std::ops::Deref for Attribute {
//     type Target = SiddhiElement;
//     fn deref(&self) -> &Self::Target { &self.siddhi_element }
// }
// impl std::ops::DerefMut for Attribute {
//     fn deref_mut(&mut self) -> &mut Self::Target { &mut self.siddhi_element }
// }

#[cfg(test)]
mod tests {
    use super::*; // Imports Attribute and Type enum

    #[test]
    fn test_attribute_creation() {
        let attr = Attribute::new("timestamp".to_string(), Type::LONG);
        assert_eq!(attr.get_name(), "timestamp");
        assert_eq!(attr.get_type(), &Type::LONG); // Compare with borrowed Type
                                                  // Check siddhi_element defaults
        assert_eq!(attr.siddhi_element.query_context_start_index, None);
        assert_eq!(attr.siddhi_element.query_context_end_index, None);
    }

    #[test]
    fn test_attribute_type_default() {
        assert_eq!(Type::default(), Type::OBJECT);
    }
}
