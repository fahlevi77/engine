// Corresponds to io.siddhi.query.api.SiddhiElement

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)] // Added Eq, Hash, Default
pub struct SiddhiElement {
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,
}

impl SiddhiElement {
    pub fn new(start_index: Option<(i32, i32)>, end_index: Option<(i32, i32)>) -> Self {
        SiddhiElement {
            query_context_start_index: start_index,
            query_context_end_index: end_index,
        }
    }
}

// The trait approach mentioned in previous subtasks for get/set methods:
// pub trait SiddhiElementTrait {
//     fn query_context_start_index(&self) -> Option<(i32, i32)>;
//     fn set_query_context_start_index(&mut self, index: Option<(i32, i32)>);
//     fn query_context_end_index(&self) -> Option<(i32, i32)>;
//     fn set_query_context_end_index(&mut self, index: Option<(i32, i32)>);
// }
// impl SiddhiElementTrait for SiddhiElement { ... }
// This is useful if other structs embed SiddhiElement and want to delegate.
// For now, direct field access is okay as they are public.
// The prompt asks to ensure structs that *use* SiddhiElement initialize it with `SiddhiElement::default()`.
// This implies that structs will *compose* SiddhiElement.
// The existing files mostly have `query_context_start_index` fields directly,
// which is fine if they also implement `SiddhiElementTrait` or provide similar methods.
// The prompt for this task asks for `element: SiddhiElement::default()` in structs,
// so I will assume composition is the target for structs that are SiddhiElements.
// This means many files will need to change from direct fields to `element: SiddhiElement`.
