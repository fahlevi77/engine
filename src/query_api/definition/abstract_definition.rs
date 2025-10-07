// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to io.eventflux.query.api.definition.AbstractDefinition
use crate::query_api::annotation::Annotation;
use crate::query_api::definition::attribute::Attribute;
use crate::query_api::eventflux_element::EventFluxElement; // Assuming Annotation is defined as per previous steps

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct AbstractDefinition {
    pub eventflux_element: EventFluxElement, // Renamed from 'element' and uses default initialization

    // AbstractDefinition fields
    pub id: String,
    pub attribute_list: Vec<Attribute>,
    // attribute_name_array is derived from attribute_list in Java,
    // so we might not need it as a separate field in Rust if we can compute it on demand.
    // has_definition_changed is a helper for attribute_name_array, may not be needed.
    pub annotations: Vec<Annotation>,
}

impl AbstractDefinition {
    pub fn new(id: String) -> Self {
        AbstractDefinition {
            eventflux_element: EventFluxElement::default(),
            id,
            attribute_list: Vec::new(),
            annotations: Vec::new(),
        }
    }

    // Getter methods
    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn get_attribute_list(&self) -> &Vec<Attribute> {
        &self.attribute_list
    }

    // TODO: Builder methods from Java for attribute and annotation
    // pub fn attribute(mut self, attribute_name: String, attribute_type: crate::query_api::definition::attribute::Type) -> Self {
    //     // TODO: checkAttribute logic
    //     self.attribute_list.push(Attribute::new(attribute_name, attribute_type));
    //     self
    // }
    // pub fn annotation(mut self, annotation: Annotation) -> Self {
    //     self.annotations.push(annotation);
    //     self
    // }
}

// AsRef and AsMut implementations are good if direct delegation is preferred over field access.
// If eventflux_element is public, direct access `my_def.eventflux_element` is also possible.
impl AsRef<EventFluxElement> for AbstractDefinition {
    fn as_ref(&self) -> &EventFluxElement {
        &self.eventflux_element
    }
}

impl AsMut<EventFluxElement> for AbstractDefinition {
    fn as_mut(&mut self) -> &mut EventFluxElement {
        &mut self.eventflux_element
    }
}
