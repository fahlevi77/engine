use crate::query_api::siddhi_element::SiddhiElement;
// Annotation is not used here as Java OnDemandQuery doesn't have annotations field.
// use crate::query_api::annotation::Annotation;
use super::{OutputEventType, OutputStream}; // Use parent module's re-exports
use crate::query_api::execution::query::input::InputStore;
use crate::query_api::execution::query::output::output_stream::{
    DeleteStreamAction, OutputStreamAction, UpdateOrInsertStreamAction, UpdateStreamAction,
};
use crate::query_api::execution::query::output::stream::UpdateSet;
use crate::query_api::execution::query::selection::Selector;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)] // Added Eq, Hash, Copy
pub enum OnDemandQueryType {
    Insert,
    Delete,
    Update,
    Select,
    UpdateOrInsert,
    Find,
}

impl Default for OnDemandQueryType {
    fn default() -> Self {
        OnDemandQueryType::Select
    } // Defaulting to Select/Find
}

#[derive(Clone, Debug, PartialEq)] // Default will be custom via new()
pub struct OnDemandQuery {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    pub input_store: Option<InputStore>,
    pub selector: Selector,
    pub output_stream: OutputStream,
    pub on_demand_query_type: Option<OnDemandQueryType>,
    // No annotations field in Java OnDemandQuery
}

impl OnDemandQuery {
    pub fn new() -> Self {
        OnDemandQuery {
            siddhi_element: SiddhiElement::default(),
            input_store: None,
            selector: Selector::new(),
            output_stream: OutputStream::default_return_stream(),
            on_demand_query_type: None, // Or Some(OnDemandQueryType::default())
        }
    }

    // Static factory from Java
    pub fn query() -> Self {
        Self::new()
    }

    // Builder methods
    pub fn from(mut self, input_store: InputStore) -> Self {
        self.input_store = Some(input_store);
        self
    }

    pub fn select(mut self, selector: Selector) -> Self {
        self.selector = selector;
        self
    }

    pub fn out_stream(mut self, output_stream: OutputStream) -> Self {
        self.output_stream = output_stream;
        if self.output_stream.get_output_event_type().is_none() {
            self.output_stream
                .set_output_event_type_if_none(OutputEventType::CurrentEvents);
        }
        self
    }

    pub fn set_type(mut self, query_type: OnDemandQueryType) -> Self {
        self.on_demand_query_type = Some(query_type);
        self
    }

    pub fn delete_by(
        mut self,
        output_table_id: String,
        on_deleting_expression: Expression,
    ) -> Self {
        let action = DeleteStreamAction {
            target_id: output_table_id,
            on_delete_expression: on_deleting_expression,
        };
        self.output_stream = OutputStream::new(
            OutputStreamAction::Delete(action),
            Some(OutputEventType::CurrentEvents),
        );
        self
    }

    pub fn update_by(mut self, output_table_id: String, on_update_expression: Expression) -> Self {
        let action = UpdateStreamAction {
            target_id: output_table_id,
            on_update_expression,
            update_set_clause: None,
        };
        self.output_stream = OutputStream::new(
            OutputStreamAction::Update(action),
            Some(OutputEventType::CurrentEvents),
        );
        self
    }

    pub fn update_by_with_set(
        mut self,
        output_table_id: String,
        update_set_attributes: UpdateSet,
        on_update_expression: Expression,
    ) -> Self {
        let action = UpdateStreamAction {
            target_id: output_table_id,
            on_update_expression,
            update_set_clause: Some(update_set_attributes),
        };
        self.output_stream = OutputStream::new(
            OutputStreamAction::Update(action),
            Some(OutputEventType::CurrentEvents),
        );
        self
    }

    pub fn update_or_insert_by(
        mut self,
        output_table_id: String,
        update_set_attributes: UpdateSet,
        on_update_expression: Expression,
    ) -> Self {
        let action = UpdateOrInsertStreamAction {
            target_id: output_table_id,
            on_update_expression,
            update_set_clause: Some(update_set_attributes),
        };
        self.output_stream = OutputStream::new(
            OutputStreamAction::UpdateOrInsert(action),
            Some(OutputEventType::CurrentEvents),
        );
        self
    }

    pub fn get_input_store(&self) -> Option<&InputStore> {
        self.input_store.as_ref()
    }

    pub fn get_selector(&self) -> &Selector {
        &self.selector
    }

    pub fn get_output_stream(&self) -> &OutputStream {
        &self.output_stream
    }

    pub fn get_type(&self) -> Option<OnDemandQueryType> {
        self.on_demand_query_type
    }
}

impl Default for OnDemandQuery {
    fn default() -> Self {
        Self::new()
    }
}

// SiddhiElement is composed. Access via self.siddhi_element.
// No direct impl of SiddhiElement trait needed on OnDemandQuery itself if using composition and public field.
// However, Java's OnDemandQuery *is* a SiddhiElement. So, if we want to pass OnDemandQuery
// where a dyn SiddhiElement is expected, it should implement the trait (by delegating).
// For now, assuming direct access to composed `siddhi_element` is sufficient, or this will be added later.
