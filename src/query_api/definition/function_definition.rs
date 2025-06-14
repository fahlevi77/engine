// Corresponds to io.siddhi.query.api.definition.FunctionDefinition
use crate::query_api::definition::attribute::Type as AttributeType;
use crate::query_api::siddhi_element::SiddhiElement;

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct FunctionDefinition {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // FunctionDefinition fields
    pub id: String,
    pub language: String,
    pub body: String,
    pub return_type: AttributeType, // Default for AttributeType::OBJECT
}

impl FunctionDefinition {
    // Constructor requiring all essential fields.
    pub fn new(id: String, language: String, body: String, return_type: AttributeType) -> Self {
        FunctionDefinition {
            siddhi_element: SiddhiElement::default(),
            id,
            language,
            body,
            return_type,
        }
    }

    // Builder-style methods from Java
    // These are now associated functions that construct a new FunctionDefinition
    // or modify an existing one if they take `mut self`.
    // The Java API implies these are builder steps on a new object.

    // Static factories for builder pattern start
    pub fn id(id: String) -> Self {
        // Starts the build with an ID
        FunctionDefinition {
            id,
            ..Default::default() // Sets siddhi_element, language, body, return_type to default
        }
    }

    // Methods to modify fields (fluent interface)
    pub fn language(mut self, language: String) -> Self {
        self.language = language;
        self
    }

    pub fn body(mut self, body: String) -> Self {
        self.body = body;
        self
    }

    // Renamed from `type` in Java
    pub fn return_type(mut self, return_type: AttributeType) -> Self {
        self.return_type = return_type;
        self
    }
}

// Removed: impl From<SiddhiElement> for FunctionDefinition

// For direct field access via Deref, if desired:
// impl std::ops::Deref for FunctionDefinition {
//     type Target = SiddhiElement;
//     fn deref(&self) -> &Self::Target { &self.siddhi_element }
// }
// impl std::ops::DerefMut for FunctionDefinition {
//     fn deref_mut(&mut self) -> &mut Self::Target { &mut self.siddhi_element }
// }
