// Corresponds to io.siddhi.query.api.expression.Variable
use crate::query_api::siddhi_element::SiddhiElement;

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct Variable {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // Variable specific fields
    pub stream_id: Option<String>,
    pub is_inner_stream: bool,
    pub stream_index: Option<i32>,
    pub function_id: Option<String>,
    pub function_index: Option<i32>,
    pub attribute_name: String,
}

impl Variable {
    // Constructor requires attribute_name, others are optional or defaulted.
    pub fn new(attribute_name: String) -> Self {
        Variable {
            siddhi_element: SiddhiElement::default(),
            attribute_name,
            stream_id: None,
            is_inner_stream: false,
            stream_index: None,
            function_id: None,
            function_index: None,
        }
    }

    // Builder methods from Java
    pub fn of_stream(mut self, stream_id: String) -> Self {
        self.stream_id = Some(stream_id);
        self.is_inner_stream = false;
        self
    }

    pub fn of_inner_stream(mut self, stream_id: String) -> Self {
        self.stream_id = Some(stream_id);
        self.is_inner_stream = true;
        self
    }

    pub fn of_stream_with_index(mut self, stream_id: String, stream_index: i32) -> Self {
        self.stream_id = Some(stream_id);
        self.stream_index = Some(stream_index);
        self.is_inner_stream = false;
        self
    }

    pub fn of_inner_stream_with_index(mut self, stream_id: String, stream_index: i32) -> Self {
        self.stream_id = Some(stream_id);
        self.stream_index = Some(stream_index);
        self.is_inner_stream = true;
        self
    }

    pub fn of_function(mut self, function_id: String) -> Self {
        self.function_id = Some(function_id);
        self
    }

    pub fn of_function_with_index(mut self, function_id: String, function_index: i32) -> Self {
        self.function_id = Some(function_id);
        self.function_index = Some(function_index);
        self
    }
}

// Constants like `Variable.LAST` (-2) can be defined here if needed.
pub const LAST_INDEX: i32 = -2;

// Deref if needed:
// impl std::ops::Deref for Variable {
//     type Target = SiddhiElement;
//     fn deref(&self) -> &Self::Target { &self.siddhi_element }
// }
// impl std::ops::DerefMut for Variable {
//     fn deref_mut(&mut self) -> &mut Self::Target { &mut self.siddhi_element }
// }
