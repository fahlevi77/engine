// siddhi_rust/src/core/event/stream/populater/stream_mapping_element.rs
#[derive(Debug, Clone)]
pub struct StreamMappingElement {
    pub from_position: usize,
    pub to_position: Option<Vec<i32>>, // Siddhi position array
}

impl StreamMappingElement {
    pub fn new(from_position: usize, to_position: Option<Vec<i32>>) -> Self {
        Self { from_position, to_position }
    }
}
