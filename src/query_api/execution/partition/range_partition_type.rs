use crate::query_api::expression::Expression;

#[derive(Clone, Debug, PartialEq)]
pub struct RangePartitionProperty {
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,
    pub partition_key: String,
    pub condition: Expression,
}

impl RangePartitionProperty {
    pub fn new(partition_key: String, condition: Expression) -> Self {
        RangePartitionProperty {
            query_context_start_index: None,
            query_context_end_index: None,
            partition_key,
            condition,
        }
    }
}

// impl EventFluxElement for RangePartitionProperty removed

#[derive(Clone, Debug, PartialEq)]
pub struct RangePartitionType {
    // This 'element' field was in the prompt, but Java RangePartitionType implements EventFluxElement directly.
    // So, query_context_start/end_index will be direct fields.
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    pub stream_id: String, // The ID of the stream that is being partitioned

    // In Java, this is `RangePartitionProperty[] rangePartitionProperties`.
    // The prompt used `intervals: Vec<(Expression, String)>`.
    // Using Vec<RangePartitionProperty> is closer to Java and retains named fields.
    pub range_partition_properties: Vec<RangePartitionProperty>,
}

impl RangePartitionType {
    pub fn new(stream_id: String, range_partition_properties: Vec<RangePartitionProperty>) -> Self {
        RangePartitionType {
            query_context_start_index: None,
            query_context_end_index: None,
            stream_id,
            range_partition_properties,
        }
    }
}

// impl EventFluxElement for RangePartitionType removed
