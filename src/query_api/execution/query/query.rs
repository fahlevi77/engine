use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::annotation::Annotation;
use crate::query_api::execution::query::input::InputStream;
use crate::query_api::execution::query::selection::Selector;
use super::{OutputStream, OutputRate, OutputEventType}; // Use parent module's re-exports
use crate::query_api::execution::execution_element::ExecutionElementTrait;


#[derive(Clone, Debug, PartialEq)] // Default will be custom or via new()
pub struct Query {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    pub input_stream: Option<InputStream>,
    pub selector: Selector,
    pub output_stream: OutputStream,
    pub output_rate: Option<OutputRate>,
    pub annotations: Vec<Annotation>,
}

impl Query {
    pub fn new() -> Self {
        Query {
            siddhi_element: SiddhiElement::default(), // Initialize composed element
            input_stream: None,
            selector: Selector::new(), // Java default
            output_stream: OutputStream::default_return_stream(), // Java default
            output_rate: None,
            annotations: Vec::new(),
        }
    }

    // Static factory method from Java
    pub fn query() -> Self {
        Self::new()
    }

    // Builder methods
    pub fn from(mut self, input_stream: InputStream) -> Self {
        self.input_stream = Some(input_stream);
        self
    }

    pub fn select(mut self, selector: Selector) -> Self {
        self.selector = selector;
        self
    }

    pub fn out_stream(mut self, output_stream: OutputStream) -> Self {
        self.output_stream = output_stream;
        self.update_output_event_type();
        self
    }

    pub fn output(mut self, output_rate: OutputRate) -> Self {
        self.output_rate = Some(output_rate);
        self.update_output_event_type();
        self
    }

    fn update_output_event_type(&mut self) {
        if self.output_stream.get_output_event_type().is_none() {
            let is_snapshot_rate = self.output_rate.as_ref().map_or(false, |r| r.is_snapshot());

            let event_type = if is_snapshot_rate {
                OutputEventType::AllEvents
            } else {
                OutputEventType::CurrentEvents
            };
            self.output_stream.set_output_event_type_if_none(event_type);
        }
    }

    pub fn annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    // Getter for annotations, needed by ExecutionElementTrait impl
    pub fn get_annotations(&self) -> &Vec<Annotation> {
        &self.annotations
    }
}

impl Default for Query {
    fn default() -> Self {
        Self::new()
    }
}

// SiddhiElement is composed, access via `self.siddhi_element.query_context_start_index` etc.
// Or implement Deref/DerefMut if desired for direct access.

// Implement ExecutionElementTrait for Query
impl ExecutionElementTrait for Query {
    fn get_annotations(&self) -> &Vec<Annotation> {
        &self.annotations
    }
}
