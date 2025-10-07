// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to io.eventflux.query.api.expression.condition.IsNull
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct IsNull {
    pub eventflux_element: EventFluxElement, // Composed EventFluxElement

    // IsNull specific fields
    pub expression: Option<Box<Expression>>,

    pub stream_id: Option<String>,
    pub stream_index: Option<i32>,
    pub is_inner_stream: bool,
    pub is_fault_stream: bool,
}

impl IsNull {
    // Constructor for `isNull(Expression expression)`
    pub fn new_with_expression(expression: Expression) -> Self {
        IsNull {
            eventflux_element: EventFluxElement::default(),
            expression: Some(Box::new(expression)),
            stream_id: None,
            stream_index: None,
            is_inner_stream: false, // Default for this path
            is_fault_stream: false, // Default for this path
        }
    }

    // Constructor for `isNullStream(String streamId, Integer streamIndex, boolean isInnerStream, boolean isFaultStream)`
    pub fn new_with_stream_details(
        stream_id: String,
        stream_index: Option<i32>,
        is_inner_stream: bool,
        is_fault_stream: bool,
    ) -> Self {
        IsNull {
            eventflux_element: EventFluxElement::default(),
            expression: None,
            stream_id: Some(stream_id),
            stream_index,
            is_inner_stream,
            is_fault_stream,
        }
    }
}
