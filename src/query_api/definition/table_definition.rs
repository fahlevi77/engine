// Corresponds to io.siddhi.query.api.definition.TableDefinition
use crate::query_api::annotation::Annotation;
use crate::query_api::definition::abstract_definition::AbstractDefinition;
use crate::query_api::definition::attribute::{Attribute, Type as AttributeType}; // Assuming Annotation is defined

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct TableDefinition {
    // Composition for inheritance from AbstractDefinition
    pub abstract_definition: AbstractDefinition,
    // TableDefinition, like StreamDefinition, doesn't add new fields in the Java version.
}

impl TableDefinition {
    // Constructor that takes an id, as per Java's `TableDefinition.id(id)`
    pub fn new(id: String) -> Self {
        TableDefinition {
            abstract_definition: AbstractDefinition::new(id),
        }
    }

    // Static factory method `id` from Java
    pub fn id(table_id: String) -> Self {
        Self::new(table_id)
    }

    // Builder-style methods, specific to TableDefinition
    pub fn attribute(mut self, attribute_name: String, attribute_type: AttributeType) -> Self {
        // TODO: Implement checkAttribute logic from AbstractDefinition or call a method on it.
        self.abstract_definition
            .attribute_list
            .push(Attribute::new(attribute_name, attribute_type));
        self
    }

    pub fn annotation(mut self, annotation: Annotation) -> Self {
        self.abstract_definition.annotations.push(annotation);
        self
    }
}

// Provide access to AbstractDefinition fields and SiddhiElement fields
impl AsRef<AbstractDefinition> for TableDefinition {
    fn as_ref(&self) -> &AbstractDefinition {
        &self.abstract_definition
    }
}

impl AsMut<AbstractDefinition> for TableDefinition {
    fn as_mut(&mut self) -> &mut AbstractDefinition {
        &mut self.abstract_definition
    }
}

// Through AbstractDefinition, can access SiddhiElement
use crate::query_api::siddhi_element::SiddhiElement;
impl AsRef<SiddhiElement> for TableDefinition {
    fn as_ref(&self) -> &SiddhiElement {
        self.abstract_definition.as_ref()
    }
}

impl AsMut<SiddhiElement> for TableDefinition {
    fn as_mut(&mut self) -> &mut SiddhiElement {
        self.abstract_definition.as_mut()
    }
}
