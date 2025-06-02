// Corresponds to io.siddhi.query.api.definition.Attribute
use crate::query_api::siddhi_element::SiddhiElement;

#[derive(Clone, Debug, PartialEq, Eq, Hash)] // Added Eq, Hash
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
    fn default() -> Self { Type::OBJECT } // Default type as per From<SiddhiElement> impl
}

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
