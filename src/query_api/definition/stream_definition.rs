// Corresponds to io.siddhi.query.api.definition.StreamDefinition
use crate::query_api::definition::abstract_definition::AbstractDefinition;
use crate::query_api::definition::attribute::{Attribute, Type as AttributeType};
use crate::query_api::annotation::Annotation; // Assuming Annotation is defined

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct StreamDefinition {
    // Composition for inheritance from AbstractDefinition
    pub abstract_definition: AbstractDefinition,
    // StreamDefinition itself doesn't add new fields in the Java version.
}

impl StreamDefinition {
    // Constructor that takes an id, as per Java's `StreamDefinition.id(streamId)`
    pub fn new(id: String) -> Self {
        StreamDefinition {
            abstract_definition: AbstractDefinition::new(id),
        }
    }

    // Static factory method `id` from Java
    pub fn id(stream_id: String) -> Self {
        Self::new(stream_id)
    }

    // Builder-style methods, specific to StreamDefinition
    pub fn attribute(mut self, attribute_name: String, attribute_type: AttributeType) -> Self {
        // TODO: Implement checkAttribute logic from AbstractDefinition or call a method on it.
        // For now, directly adding. Consider potential duplicates.
        // This logic should ideally be on AbstractDefinition itself.
        self.abstract_definition.attribute_list.push(Attribute::new(attribute_name, attribute_type));
        self
    }

    pub fn annotation(mut self, annotation: Annotation) -> Self {
        self.abstract_definition.annotations.push(annotation);
        self
    }

    // The `clone()` method from Java is handled by `#[derive(Clone)]`.
}

// Provide access to AbstractDefinition fields and SiddhiElement fields
// These are useful for treating StreamDefinition polymorphically if needed.
impl AsRef<AbstractDefinition> for StreamDefinition {
    fn as_ref(&self) -> &AbstractDefinition {
        &self.abstract_definition
    }
}

impl AsMut<AbstractDefinition> for StreamDefinition {
    fn as_mut(&mut self) -> &mut AbstractDefinition {
        &mut self.abstract_definition
    }
}

// Through AbstractDefinition, can access SiddhiElement
use crate::query_api::siddhi_element::SiddhiElement;
impl AsRef<SiddhiElement> for StreamDefinition {
    fn as_ref(&self) -> &SiddhiElement {
        // Accessing SiddhiElement composed within AbstractDefinition
        self.abstract_definition.as_ref()
    }
}

impl AsMut<SiddhiElement> for StreamDefinition {
    fn as_mut(&mut self) -> &mut SiddhiElement {
        self.abstract_definition.as_mut()
    }
}
