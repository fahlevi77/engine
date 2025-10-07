// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to io.eventflux.query.api.definition.TableDefinition
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
        // Check for duplicate attribute names and warn
        if self
            .abstract_definition
            .attribute_list
            .iter()
            .any(|attr| attr.get_name() == &attribute_name)
        {
            eprintln!(
                "Warning: Duplicate attribute '{}' in table definition",
                attribute_name
            );
        }

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

// Provide access to AbstractDefinition fields and EventFluxElement fields
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

// Through AbstractDefinition, can access EventFluxElement
use crate::query_api::eventflux_element::EventFluxElement;
impl AsRef<EventFluxElement> for TableDefinition {
    fn as_ref(&self) -> &EventFluxElement {
        self.abstract_definition.as_ref()
    }
}

impl AsMut<EventFluxElement> for TableDefinition {
    fn as_mut(&mut self) -> &mut EventFluxElement {
        self.abstract_definition.as_mut()
    }
}
