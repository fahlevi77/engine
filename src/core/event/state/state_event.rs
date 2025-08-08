// siddhi_rust/src/core/event/state/state_event.rs
// Corresponds to io.siddhi.core.event.state.StateEvent
use crate::core::event::complex_event::{ComplexEvent, ComplexEventType};
use crate::core::event::stream::StreamEvent; // StateEvent holds StreamEvents
use crate::core::event::value::AttributeValue;
// use crate::core::util::siddhi_constants::{
//     BEFORE_WINDOW_DATA_INDEX, CURRENT, LAST, ON_AFTER_WINDOW_DATA_INDEX, OUTPUT_DATA_INDEX,
//     STATE_OUTPUT_DATA_INDEX, STREAM_ATTRIBUTE_INDEX_IN_TYPE, STREAM_ATTRIBUTE_TYPE_INDEX,
//     STREAM_EVENT_CHAIN_INDEX, STREAM_EVENT_INDEX_IN_CHAIN,
// }; // TODO: Will be used when implementing state event operations
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_STATE_EVENT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Default, Serialize, Deserialize)] // Default is placeholder
pub struct StateEvent {
    // Fields from Java StateEvent
    pub stream_events: Vec<Option<StreamEvent>>, // Java: StreamEvent[], can have nulls
    pub timestamp: i64,
    pub event_type: ComplexEventType, // Corresponds to type field
    pub output_data: Option<Vec<AttributeValue>>, // Corresponds to outputData field

    // For ComplexEvent linked list
    #[serde(default, skip_serializing, skip_deserializing)]
    pub next: Option<Box<dyn ComplexEvent>>,

    // Java StateEvent has an 'id' field too (long)
    pub id: u64, // Using u64 to match Event.id, though Java StateEvent.id is not static atomic
}

impl Clone for StateEvent {
    fn clone(&self) -> Self {
        StateEvent {
            stream_events: self.stream_events.clone(),
            timestamp: self.timestamp,
            event_type: self.event_type,
            output_data: self.output_data.clone(),
            next: self
                .next
                .as_ref()
                .map(|n| crate::core::event::complex_event::clone_box_complex_event(n.as_ref())),
            id: self.id,
        }
    }
}

impl StateEvent {
    pub fn clone_without_next(&self) -> Self {
        StateEvent {
            stream_events: self.stream_events.clone(),
            timestamp: self.timestamp,
            event_type: self.event_type,
            output_data: self.output_data.clone(),
            next: None,
            id: self.id,
        }
    }
    pub fn new(stream_events_size: usize, output_size: usize) -> Self {
        // Initialize stream_events with None
        let mut stream_events_vec = Vec::with_capacity(stream_events_size);
        for _ in 0..stream_events_size {
            stream_events_vec.push(None);
        }

        Self {
            stream_events: stream_events_vec,
            timestamp: -1, // Default timestamp
            event_type: ComplexEventType::default(),
            output_data: if output_size > 0 {
                Some(vec![AttributeValue::default(); output_size])
            } else {
                None
            },
            next: None,
            id: NEXT_STATE_EVENT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
    pub fn get_stream_event(&self, position: usize) -> Option<&StreamEvent> {
        self.stream_events.get(position)?.as_ref()
    }

    pub fn get_stream_events(&self) -> &Vec<Option<StreamEvent>> {
        &self.stream_events
    }

    pub fn set_event(&mut self, position: usize, event: StreamEvent) {
        if position < self.stream_events.len() {
            self.stream_events[position] = Some(event);
        }
    }

    pub fn add_event(&mut self, position: usize, stream_event: StreamEvent) {
        // Simplified: just append if slot empty, otherwise replace existing next chain head.
        if position >= self.stream_events.len() {
            return;
        }
        match &mut self.stream_events[position] {
            None => self.stream_events[position] = Some(stream_event),
            Some(existing) => existing.next = Some(Box::new(stream_event)),
        }
    }

    /// Retrieve a stream event based on Siddhi position array logic.
    pub fn get_stream_event_by_position(&self, position: &[i32]) -> Option<&StreamEvent> {
        use crate::core::util::siddhi_constants::{
            CURRENT, LAST, STREAM_EVENT_CHAIN_INDEX, STREAM_EVENT_INDEX_IN_CHAIN,
        };
        let mut stream_event = self
            .stream_events
            .get(*position.get(STREAM_EVENT_CHAIN_INDEX)? as usize)?
            .as_ref()?;
        let mut current = stream_event as &dyn ComplexEvent;
        let idx = *position.get(STREAM_EVENT_INDEX_IN_CHAIN)?;
        if idx >= 0 {
            for _ in 1..=idx {
                current = current.get_next()?;
                stream_event = current.as_any().downcast_ref::<StreamEvent>()?;
            }
        } else if idx == CURRENT {
            while let Some(next) = current.get_next() {
                current = next;
                stream_event = current.as_any().downcast_ref::<StreamEvent>()?;
            }
        } else if idx == LAST {
            current.get_next()?;
            while let Some(next_next) = current.get_next().and_then(|n| n.get_next()) {
                current = current.get_next()?;
                stream_event = current.as_any().downcast_ref::<StreamEvent>()?;
                if next_next.get_next().is_none() {
                    break;
                }
            }
        } else {
            let mut list = Vec::new();
            let mut tmp: Option<&dyn ComplexEvent> = Some(current);
            while let Some(ev) = tmp {
                list.push(ev.as_any().downcast_ref::<StreamEvent>()?);
                tmp = ev.get_next();
            }
            let index = list.len() as i32 + idx;
            if index < 0 {
                return None;
            }
            stream_event = list.get(index as usize)?;
        }
        Some(stream_event)
    }

    pub fn remove_last_event(&mut self, position: usize) {
        if position >= self.stream_events.len() {
            return;
        }
        if let Some(ref mut event) = self.stream_events[position] {
            // Simplified: drop the chain entirely
            event.next = None;
        }
    }
}

// Inherent methods for StateEvent
impl StateEvent {
    pub fn set_output_data_at_idx(
        &mut self,
        value: AttributeValue,
        index: usize,
    ) -> Result<(), String> {
        if let Some(data) = self.output_data.as_mut() {
            if index < data.len() {
                data[index] = value;
                Ok(())
            } else {
                Err(format!(
                    "Index {} out of bounds for output_data with len {}",
                    index,
                    data.len()
                ))
            }
        } else if index == 0 {
            let mut new_data = vec![AttributeValue::default(); index + 1];
            new_data[index] = value;
            self.output_data = Some(new_data);
            Ok(())
        } else {
            Err("output_data is None and index is not 0, cannot set value".to_string())
        }
    }

    pub fn set_output_data_vec(&mut self, data: Option<Vec<AttributeValue>>) {
        self.output_data = data;
    }

    pub fn get_output_data(&self) -> Option<&[AttributeValue]> {
        self.output_data.as_deref()
    }

    pub fn get_attribute(&self, position: &[i32]) -> Option<&AttributeValue> {
        use crate::core::util::siddhi_constants::{
            BEFORE_WINDOW_DATA_INDEX, ON_AFTER_WINDOW_DATA_INDEX, OUTPUT_DATA_INDEX,
            STATE_OUTPUT_DATA_INDEX, STREAM_ATTRIBUTE_INDEX_IN_TYPE, STREAM_ATTRIBUTE_TYPE_INDEX,
        };
        if *position.get(STREAM_ATTRIBUTE_TYPE_INDEX)? as usize == STATE_OUTPUT_DATA_INDEX {
            return self
                .output_data
                .as_ref()
                .and_then(|d| d.get(*position.get(STREAM_ATTRIBUTE_INDEX_IN_TYPE)? as usize));
        }
        let se = self.get_stream_event_by_position(position)?;
        match *position.get(STREAM_ATTRIBUTE_TYPE_INDEX)? as usize {
            BEFORE_WINDOW_DATA_INDEX => se
                .before_window_data
                .get(*position.get(STREAM_ATTRIBUTE_INDEX_IN_TYPE)? as usize),
            OUTPUT_DATA_INDEX => se
                .output_data
                .as_ref()?
                .get(*position.get(STREAM_ATTRIBUTE_INDEX_IN_TYPE)? as usize),
            ON_AFTER_WINDOW_DATA_INDEX => se
                .on_after_window_data
                .get(*position.get(STREAM_ATTRIBUTE_INDEX_IN_TYPE)? as usize),
            _ => None,
        }
    }

    pub fn set_attribute(&mut self, value: AttributeValue, position: &[i32]) -> Result<(), String> {
        use crate::core::util::siddhi_constants::{
            BEFORE_WINDOW_DATA_INDEX, ON_AFTER_WINDOW_DATA_INDEX, OUTPUT_DATA_INDEX,
            STATE_OUTPUT_DATA_INDEX, STREAM_ATTRIBUTE_INDEX_IN_TYPE, STREAM_ATTRIBUTE_TYPE_INDEX,
        };
        if *position.get(STREAM_ATTRIBUTE_TYPE_INDEX).ok_or("pos")? as usize
            == STATE_OUTPUT_DATA_INDEX
        {
            if let Some(ref mut vec) = self.output_data {
                let idx = *position.get(STREAM_ATTRIBUTE_INDEX_IN_TYPE).ok_or("pos")? as usize;
                if idx < vec.len() {
                    vec[idx] = value;
                    return Ok(());
                } else {
                    return Err("index out of bounds".into());
                }
            } else {
                return Err("output_data is None".into());
            }
        }
        let se_position = *position
            .get(crate::core::util::siddhi_constants::STREAM_EVENT_CHAIN_INDEX)
            .ok_or("pos")? as usize;
        if se_position >= self.stream_events.len() {
            return Err("stream position out of bounds".into());
        }
        let se = self.stream_events[se_position]
            .as_mut()
            .ok_or("no stream event")?;
        let idx = *position.get(STREAM_ATTRIBUTE_INDEX_IN_TYPE).ok_or("pos")? as usize;
        match *position.get(STREAM_ATTRIBUTE_TYPE_INDEX).ok_or("pos")? as usize {
            BEFORE_WINDOW_DATA_INDEX => {
                if idx < se.before_window_data.len() {
                    se.before_window_data[idx] = value;
                    Ok(())
                } else {
                    Err("index out".into())
                }
            }
            OUTPUT_DATA_INDEX => {
                if let Some(ref mut out) = se.output_data {
                    if idx < out.len() {
                        out[idx] = value;
                        Ok(())
                    } else {
                        Err("index out".into())
                    }
                } else {
                    Err("output_data None".into())
                }
            }
            ON_AFTER_WINDOW_DATA_INDEX => {
                if idx < se.on_after_window_data.len() {
                    se.on_after_window_data[idx] = value;
                    Ok(())
                } else {
                    Err("index out".into())
                }
            }
            _ => Err("invalid type".into()),
        }
    }
}

impl ComplexEvent for StateEvent {
    fn get_next(&self) -> Option<&dyn ComplexEvent> {
        self.next.as_deref()
    }

    fn set_next(
        &mut self,
        next_event: Option<Box<dyn ComplexEvent>>,
    ) -> Option<Box<dyn ComplexEvent>> {
        let old_next = self.next.take();
        self.next = next_event;
        old_next
    }

    fn mut_next_ref_option(&mut self) -> &mut Option<Box<dyn ComplexEvent>> {
        &mut self.next
    }

    fn get_output_data(&self) -> Option<&[AttributeValue]> {
        self.output_data.as_deref()
    }

    fn set_output_data(&mut self, data: Option<Vec<AttributeValue>>) {
        self.output_data = data;
    }

    fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    fn set_timestamp(&mut self, timestamp: i64) {
        self.timestamp = timestamp;
    }

    fn get_event_type(&self) -> ComplexEventType {
        self.event_type
    }

    fn set_event_type(&mut self, event_type: ComplexEventType) {
        self.event_type = event_type;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
