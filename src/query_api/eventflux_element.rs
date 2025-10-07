// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to io.eventflux.query.api.EventFluxElement

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)] // Added Eq, Hash, Default
pub struct EventFluxElement {
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,
}

impl EventFluxElement {
    pub fn new(start_index: Option<(i32, i32)>, end_index: Option<(i32, i32)>) -> Self {
        EventFluxElement {
            query_context_start_index: start_index,
            query_context_end_index: end_index,
        }
    }
}

// The trait approach mentioned in previous subtasks for get/set methods:
// pub trait EventFluxElementTrait {
//     fn query_context_start_index(&self) -> Option<(i32, i32)>;
//     fn set_query_context_start_index(&mut self, index: Option<(i32, i32)>);
//     fn query_context_end_index(&self) -> Option<(i32, i32)>;
//     fn set_query_context_end_index(&mut self, index: Option<(i32, i32)>);
// }
// impl EventFluxElementTrait for EventFluxElement { ... }
// This is useful if other structs embed EventFluxElement and want to delegate.
// For now, direct field access is okay as they are public.
// The prompt asks to ensure structs that *use* EventFluxElement initialize it with `EventFluxElement::default()`.
// This implies that structs will *compose* EventFluxElement.
// The existing files mostly have `query_context_start_index` fields directly,
// which is fine if they also implement `EventFluxElementTrait` or provide similar methods.
// The prompt for this task asks for `element: EventFluxElement::default()` in structs,
// so I will assume composition is the target for structs that are EventFluxElements.
// This means many files will need to change from direct fields to `element: EventFluxElement`.
