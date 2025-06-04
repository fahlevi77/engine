// siddhi_rust/src/core/executor/variable_expression_executor.rs
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent; // Trait
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::attribute::Type as ApiAttributeType; // Import Type enum
// Removed QueryApiVariable, SiddhiAppContext, VariablePosition, EventDataArrayType from direct use here
// as the parser will now do the resolution and provide a direct index.

/// Executor that retrieves a variable's value from an event. Current impl is simplified.
#[derive(Debug, Clone)]
pub struct VariableExpressionExecutor {
    // Index of the attribute in the event's primary data array.
    // This simplification assumes a single data array per ComplexEvent relevant for this VEE.
    // The actual array (output_data, before_window_data, etc.) and stream (for StateEvent)
    // must be determined by the parser when creating this index.
    pub attribute_index_in_data: usize,
    pub return_type: ApiAttributeType,
    pub attribute_name_for_debug: String, // For easier debugging
}

impl VariableExpressionExecutor {
    pub fn new(
        attribute_index_in_data: usize,
        return_type: ApiAttributeType,
        attribute_name_for_debug: String,
    ) -> Self {
        Self {
            attribute_index_in_data,
            return_type,
            attribute_name_for_debug,
        }
    }
}

impl ExpressionExecutor for VariableExpressionExecutor {
    fn execute(&self, event_opt: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let complex_event = event_opt?;

        // Simplified logic: Assumes the ComplexEvent's `get_output_data()` is the relevant array.
        // This is a major simplification and will need to be made more sophisticated to handle:
        // - Different event types (StreamEvent vs StateEvent)
        // - Different data arrays within an event (beforeWindowData, onAfterWindowData, outputData for StreamEvent)
        // - Correct indexing into StateEvent's StreamEvent array.
        // - Accessing attributes from event tables or stores (AttributeDynamicResolveType).
        // The `ExpressionParser` would need to create different kinds of VEEs or pass more detailed
        // position information for this `execute` method to be truly general.
        // For now, this matches the prompt's simplified VEE.
        // MODIFICATION: Prioritize before_window_data for StreamEvents
        if let Some(stream_event) = complex_event.as_any().downcast_ref::<crate::core::event::stream::stream_event::StreamEvent>() {
            // For StreamEvent, try to access before_window_data first.
            // This is typical for filters or expressions operating on incoming data.
            if self.attribute_index_in_data < stream_event.before_window_data.len() {
                return stream_event.before_window_data.get(self.attribute_index_in_data).cloned();
            }
            // Fallback for StreamEvent or if index is out of bounds for before_window_data:
            // This could be an else if to check on_after_window_data or output_data based on context.
            // For now, if not in before_window_data, try output_data (as original logic)
            // This part needs to be very clear based on where the VEE is used (filter vs. projection)
            // For the test_simple_filter_projection_query, filter variables are from input,
            // projection variables are also from input (relative to SelectProcessor).
            // So, `before_window_data` is likely the primary source for StreamEvents.
            // If it's a projection operating *after* some processing has put data into output_data,
            // then output_data would be the source.
            // Let's assume for now that if it's a StreamEvent, we expect it in before_window_data
            // for this VEE. If not found, it's an error or unexpected state for this VEE's role.
            // So, removing the fallback to output_data for StreamEvent to make its role clearer.
            // If specific VEEs need to access output_data of a StreamEvent, they might need a different setup.
            return None; // Not found in before_window_data or index out of bounds
        } else {
            // For other ComplexEvent types (e.g. StateEvent, or if downcast fails),
            // fall back to original output_data logic.
            // This part also needs careful consideration for StateEvents.
            if let Some(output_data) = complex_event.get_output_data() {
                return output_data.get(self.attribute_index_in_data).cloned();
            }
        }
        None
    }

    fn get_return_type(&self) -> ApiAttributeType {
        self.return_type
    }

     fn clone_executor(&self, _siddhi_app_context: &std::sync::Arc<crate::core::config::siddhi_app_context::SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
         // If siddhi_app_context was stored on self, it would be cloned:
         // siddhi_app_context: self.siddhi_app_context.as_ref().map(Arc::clone),
         Box::new(self.clone())
     }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::complex_event::{ComplexEvent, ComplexEventType};
    use crate::core::event::stream::stream_event::StreamEvent; // Using StreamEvent as a concrete ComplexEvent
    use crate::core::event::value::AttributeValue;
    // ApiAttributeType is imported in the outer scope
    use std::sync::Arc;
    use crate::core::config::siddhi_app_context::SiddhiAppContext;


    // Mock ComplexEvent or use StreamEvent for testing
    // This StreamEvent has its output_data set for testing VEE.
    fn create_test_stream_event_with_output_data(output_data_vec: Vec<AttributeValue>) -> StreamEvent {
        let mut se = StreamEvent::new(0, 0, 0, output_data_vec.len()); // ts, before, onafter, output_len
        se.output_data = Some(output_data_vec);
        se
    }

    // New helper for testing VEE with before_window_data
    fn create_test_stream_event_with_before_window_data(data_array: Vec<AttributeValue>) -> StreamEvent {
        let mut se = StreamEvent::new(0, data_array.len(), 0, 0); // timestamp, before_len, on_after_len, output_len
        se.before_window_data = data_array;
        se
    }

    // Copied helper for testing if SiddhiAppContext is needed for clone_executor
    // This should ideally be in a central test_utils module.
    impl SiddhiAppContext {
        pub fn default_for_testing() -> Self {
            use crate::core::config::siddhi_context::SiddhiContext;
            use crate::query_api::SiddhiApp as ApiSiddhiApp;

            Self::new(
                Arc::new(SiddhiContext::default()),
                "test_app_ctx_for_vee".to_string(),
                Arc::new(ApiSiddhiApp::new("test_api_app_for_vee".to_string())),
                String::new()
            )
        }
    }

    #[test]
    fn test_variable_access_from_before_window_data_string() { // Renamed and adapted
        let exec = VariableExpressionExecutor::new(
            0, // attribute_index_in_data (refers to index in before_window_data)
            ApiAttributeType::STRING,
            "test_attr".to_string()
        );
        let event_data = vec![AttributeValue::String("val1".to_string())];
        // Use the new helper that populates before_window_data
        let test_event = create_test_stream_event_with_before_window_data(event_data);

        let result = exec.execute(Some(&test_event as &dyn ComplexEvent));
        assert_eq!(result, Some(AttributeValue::String("val1".to_string())));
    }

    #[test]
    fn test_variable_access_from_before_window_data_int_out_of_bounds() { // Renamed and adapted
        let exec = VariableExpressionExecutor::new(
            1, // attribute_index_in_data (out of bounds for event below)
            ApiAttributeType::INT,
            "test_attr_int".to_string()
        );
        let event_data = vec![AttributeValue::Int(123)]; // Only one element at index 0
        // Use the new helper that populates before_window_data
        let test_event = create_test_stream_event_with_before_window_data(event_data);

        let result = exec.execute(Some(&test_event as &dyn ComplexEvent));
        // Now expects None because it only checks before_window_data for StreamEvent
        // and the index is out of bounds for that array.
        assert_eq!(result, None);
    }

    #[test]
    fn test_variable_access_output_data_for_non_stream_event_fallback() {
        // This test checks the fallback behavior for non-StreamEvent types (or failed downcast)
        // For this, we need a mock ComplexEvent that is not a StreamEvent but has output_data.
        // Or, we assume that if it's not a StreamEvent, it uses get_output_data().
        // The current code will use get_output_data() if the downcast to StreamEvent fails.
        // We can simulate this by using a StreamEvent but relying on the VEE to *not* find it
        // in before_window_data if we make before_window_data empty, and then it would hit the else branch.
        // However, the modified VEE execute for StreamEvent now returns None if not in before_window_data.
        // So, to test the `else` branch of `if let Some(stream_event) = ...`, we need a non-StreamEvent.
        // Let's create a simple mock for that.

        struct MockComplexEvent {
            output_data: Option<Vec<AttributeValue>>,
        }
        impl ComplexEvent for MockComplexEvent {
            fn get_id(&self) -> i64 { 0 }
            fn set_id(&mut self, _id: i64) {}
            fn get_timestamp(&self) -> i64 { 0 }
            fn set_timestamp(&mut self, _timestamp: i64) {}
            fn get_output_data(&self) -> Option<&Vec<AttributeValue>> { self.output_data.as_ref() }
            fn set_output_data(&mut self, _data: Option<Vec<AttributeValue>>) {}
            fn get_type(&self) -> ComplexEventType { ComplexEventType::Current }
            fn set_type(&mut self, _event_type: ComplexEventType) {}
            fn get_next(&self) -> Option<*mut dyn ComplexEvent> { None }
            fn set_next(&mut self, _event: Option<*mut dyn ComplexEvent>) {}
            fn is_expired(&self) -> bool { false }
            fn set_expired(&mut self, _is_expired: bool) {}
            fn as_any(&self) -> &dyn std::any::Any { self }
        }

        let exec = VariableExpressionExecutor::new(
            0, ApiAttributeType::STRING, "fallback_attr".to_string()
        );
        let mock_event_data = vec![AttributeValue::String("fallback_val".to_string())];
        let mock_event = MockComplexEvent { output_data: Some(mock_event_data) };

        let result = exec.execute(Some(&mock_event as &dyn ComplexEvent));
        assert_eq!(result, Some(AttributeValue::String("fallback_val".to_string())));
    }

    #[test]
    fn test_variable_access_no_event() {
        let exec = VariableExpressionExecutor::new(
            0, ApiAttributeType::STRING, "test_attr".to_string()
        );
        let result = exec.execute(None);
        assert_eq!(result, None);
    }

     #[test]
    fn test_variable_clone() { // This test needs to be re-evaluated based on new execute logic
        let exec = VariableExpressionExecutor::new(
            0, ApiAttributeType::STRING, "test_attr_clone".to_string()
        );
        let app_ctx_placeholder = Arc::new(SiddhiAppContext::default_for_testing());
        let cloned_exec = exec.clone_executor(&app_ctx_placeholder);

        assert_eq!(cloned_exec.get_return_type(), ApiAttributeType::STRING);

        // Test with before_window_data
        let event_data_before = vec![AttributeValue::String("clone_val_before".to_string())];
        let test_event_before = create_test_stream_event_with_before_window_data(event_data_before);
        let result_before = cloned_exec.execute(Some(&test_event_before as &dyn ComplexEvent));
        assert_eq!(result_before, Some(AttributeValue::String("clone_val_before".to_string())));
    }
}
