use crate::core::event::Event;
use crate::core::util::serialization::{from_bytes, to_bytes};

pub fn event_to_bytes(event: &Event) -> Result<Vec<u8>, bincode::Error> {
    to_bytes(event)
}

pub fn event_from_bytes(bytes: &[u8]) -> Result<Event, bincode::Error> {
    from_bytes(bytes)
}
