use super::range_partition_type::RangePartitionType;
use super::value_partition_type::ValuePartitionType;
use crate::query_api::siddhi_element::SiddhiElement;


#[derive(Clone, Debug, PartialEq)]
pub enum PartitionTypeVariant {
    Value(ValuePartitionType),
    Range(RangePartitionType),
}

// In Java, PartitionType is an interface.
// This struct wraps the variant and implements common traits/functionality.
#[derive(Clone, Debug, PartialEq)]
pub struct PartitionType {
    // Common SiddhiElement context for the PartitionType itself
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    pub variant: PartitionTypeVariant,
}

impl PartitionType {
    // Constructors for value and range types
    pub fn new_value(value_type: ValuePartitionType) -> Self {
        // Context for PartitionType itself might be distinct from the inner variant's context,
        // or it could be derived/copied. For now, initializing as None.
        PartitionType {
            query_context_start_index: None, // Or copy from value_type if appropriate
            query_context_end_index: None,   // Or copy from value_type if appropriate
            variant: PartitionTypeVariant::Value(value_type)
        }
    }

    pub fn new_range(range_type: RangePartitionType) -> Self {
        PartitionType {
            query_context_start_index: None, // Or copy from range_type
            query_context_end_index: None,   // Or copy from range_type
            variant: PartitionTypeVariant::Range(range_type)
        }
    }

    // getStreamId() from Java's PartitionType interface
    pub fn get_stream_id(&self) -> &str {
        match &self.variant {
            PartitionTypeVariant::Value(v) => &v.stream_id,
            PartitionTypeVariant::Range(r) => &r.stream_id,
        }
    }
}

// Implement SiddhiElement for the wrapper PartitionType struct.
// It could potentially draw its context from its variant, or have its own.
// The Java interface implies PartitionType *is* a SiddhiElement.
impl SiddhiElement for PartitionType {
    fn query_context_start_index(&self) -> Option<(i32,i32)> {
        // Prefer own context, but could fall back to variant's if design requires
        self.query_context_start_index.or_else(|| match &self.variant {
            PartitionTypeVariant::Value(v) => v.query_context_start_index(),
            PartitionTypeVariant::Range(r) => r.query_context_start_index(),
        })
    }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) {
        self.query_context_start_index = index;
        // Optionally propagate to variant if they should share context:
        // match &mut self.variant {
        //     PartitionTypeVariant::Value(v) => v.set_query_context_start_index(index),
        //     PartitionTypeVariant::Range(r) => r.set_query_context_start_index(index),
        // }
    }
    fn query_context_end_index(&self) -> Option<(i32,i32)> {
        self.query_context_end_index.or_else(|| match &self.variant {
            PartitionTypeVariant::Value(v) => v.query_context_end_index(),
            PartitionTypeVariant::Range(r) => r.query_context_end_index(),
        })
    }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) {
        self.query_context_end_index = index;
        // Optionally propagate
    }
}
