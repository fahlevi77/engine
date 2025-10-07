// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq)]
pub struct ValuePartitionType {
    // This 'element' field was in the prompt, but Java ValuePartitionType implements EventFluxElement directly.
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    pub stream_id: String, // The ID of the stream that is being partitioned
    pub expression: Expression, // The expression evaluated to get the partition key for an event
}

impl ValuePartitionType {
    pub fn new(stream_id: String, expression: Expression) -> Self {
        ValuePartitionType {
            query_context_start_index: None,
            query_context_end_index: None,
            stream_id,
            expression,
        }
    }
}

// impl EventFluxElement for ValuePartitionType removed
