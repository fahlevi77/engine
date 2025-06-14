// Corresponds to io.siddhi.query.api.definition.StreamDefinition
use crate::query_api::annotation::Annotation;
use crate::query_api::definition::abstract_definition::AbstractDefinition;
use crate::query_api::definition::attribute::{Attribute, Type as AttributeType}; // Assuming Annotation is defined

/// Defines a stream with a unique ID and a list of attributes.
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
        self.abstract_definition
            .attribute_list
            .push(Attribute::new(attribute_name, attribute_type));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_api::definition::attribute::{Attribute, Type as AttributeType};

    #[test]
    fn test_stream_definition_creation_and_attributes() {
        let stream_def = StreamDefinition::new("InputStream".to_string())
            .attribute("userID".to_string(), AttributeType::STRING)
            .attribute("value".to_string(), AttributeType::INT);

        assert_eq!(stream_def.abstract_definition.get_id(), "InputStream");

        let attributes = stream_def.abstract_definition.get_attribute_list();
        assert_eq!(attributes.len(), 2);

        assert_eq!(attributes[0].get_name(), "userID");
        assert_eq!(attributes[0].get_type(), &AttributeType::STRING);

        assert_eq!(attributes[1].get_name(), "value");
        assert_eq!(attributes[1].get_type(), &AttributeType::INT);

        // Also check default SiddhiElement from composed AbstractDefinition
        assert_eq!(
            stream_def
                .abstract_definition
                .siddhi_element
                .query_context_start_index,
            None
        );
    }

    #[test]
    fn test_stream_definition_id_factory() {
        let stream_def = StreamDefinition::id("MyStream".to_string());
        assert_eq!(stream_def.abstract_definition.get_id(), "MyStream");
    }

    #[test]
    fn test_stream_definition_annotations() {
        use crate::query_api::annotation::Annotation; // Assuming Annotation is defined
        let annotation = Annotation::new("TestAnnotation".to_string());
        let stream_def =
            StreamDefinition::new("AnnotatedStream".to_string()).annotation(annotation.clone()); // Assuming Annotation has clone

        assert_eq!(stream_def.abstract_definition.annotations.len(), 1);
        if let Some(ann) = stream_def.abstract_definition.annotations.first() {
            // Assuming Annotation has a get_name() or public name field
            // Let's assume Annotation has a name field for now or a getter
            // For this test, I'll need to check annotation.rs to confirm.
            // If Annotation::name is private, this test would need adjustment or Annotation would need a getter.
            // Based on previous work, Annotation has a public 'name' field.
            assert_eq!(ann.name, "TestAnnotation");
        } else {
            panic!("Annotation not found");
        }
    }
}
