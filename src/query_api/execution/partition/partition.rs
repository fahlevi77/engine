use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::annotation::Annotation; // Using actual Annotation
use super::partition_type::PartitionType;
use crate::query_api::execution::query::Query; // Actual Query
use std::collections::HashMap;

// For builder methods:
use crate::query_api::expression::Expression;
use super::range_partition_type::{RangePartitionProperty, RangePartitionType};
use super::value_partition_type::ValuePartitionType;


#[derive(Clone, Debug, PartialEq)]
pub struct Partition {
    // SiddhiElement fields for Partition itself
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    // Partition fields from Java
    pub partition_type_map: HashMap<String, PartitionType>, // Key is streamId being partitioned
    pub query_list: Vec<Query>,
    pub annotations: Vec<Annotation>,
    // queryNameList from Java is for validation, might not be needed as a field in Rust
}

impl Partition {
    // Constructor
    pub fn new() -> Self {
        Partition {
            query_context_start_index: None,
            query_context_end_index: None,
            partition_type_map: HashMap::new(),
            query_list: Vec::new(),
            annotations: Vec::new(),
        }
    }

    // Static factory `partition()` from Java
    pub fn partition() -> Self {
        Self::new()
    }

    // Static factory `range()` from Java Partition for creating RangePartitionProperty
    pub fn range_property(partition_key: String, condition: Expression) -> RangePartitionProperty {
        RangePartitionProperty::new(partition_key, condition)
    }

    // Builder method `with(String streamId, Expression expression)`
    pub fn with_value_partition(mut self, stream_id: String, expression: Expression) -> Self {
        let value_partition = ValuePartitionType::new(stream_id.clone(), expression);
        // TODO: Add checkDuplicateDefinition logic from Java's addPartitionType if necessary
        self.partition_type_map.insert(stream_id, PartitionType::new_value(value_partition));
        self
    }

    // Builder method `with(String streamId, RangePartitionProperty... rangePartitionProperties)`
    pub fn with_range_partition(mut self, stream_id: String, range_props: Vec<RangePartitionProperty>) -> Self {
        let range_partition = RangePartitionType::new(stream_id.clone(), range_props);
        // TODO: Add checkDuplicateDefinition logic
        self.partition_type_map.insert(stream_id, PartitionType::new_range(range_partition));
        self
    }

    // Builder method `with(PartitionType partitionType)`
    pub fn with_partition_type(mut self, partition_type: PartitionType) -> Self {
        let stream_id = partition_type.get_stream_id().to_string();
         // TODO: Add checkDuplicateDefinition logic
        self.partition_type_map.insert(stream_id, partition_type);
        self
    }

    // Builder method `addQuery(Query query)`
    pub fn add_query(mut self, query: Query) -> Self {
        // TODO: Add validation logic for duplicate query names (queryNameList in Java)
        // This would involve checking query.annotations for a name.
        self.query_list.push(query);
        self
    }

    // Builder method `annotation(Annotation annotation)`
    pub fn annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }
}

impl Default for Partition {
    fn default() -> Self {
        Self::new()
    }
}

impl SiddhiElement for Partition {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}

// Implement ExecutionElement for Partition (conceptual, actual trait defined elsewhere)
use crate::query_api::execution::execution_element::ExecutionElementTrait;
impl ExecutionElementTrait for Partition {
    fn get_annotations(&self) -> &Vec<Annotation> {
        &self.annotations
    }
}
