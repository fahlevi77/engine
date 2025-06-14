use crate::core::event::{value::AttributeValue, Event};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum AttrSer {
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Null,
}

impl From<&AttributeValue> for AttrSer {
    fn from(av: &AttributeValue) -> Self {
        match av {
            AttributeValue::String(s) => AttrSer::String(s.clone()),
            AttributeValue::Int(i) => AttrSer::Int(*i),
            AttributeValue::Long(l) => AttrSer::Long(*l),
            AttributeValue::Float(f) => AttrSer::Float(*f),
            AttributeValue::Double(d) => AttrSer::Double(*d),
            AttributeValue::Bool(b) => AttrSer::Bool(*b),
            _ => AttrSer::Null,
        }
    }
}

impl From<AttrSer> for AttributeValue {
    fn from(asr: AttrSer) -> Self {
        match asr {
            AttrSer::String(s) => AttributeValue::String(s),
            AttrSer::Int(i) => AttributeValue::Int(i),
            AttrSer::Long(l) => AttributeValue::Long(l),
            AttrSer::Float(f) => AttributeValue::Float(f),
            AttrSer::Double(d) => AttributeValue::Double(d),
            AttrSer::Bool(b) => AttributeValue::Bool(b),
            AttrSer::Null => AttributeValue::Null,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct EventSer {
    id: u64,
    timestamp: i64,
    data: Vec<AttrSer>,
    is_expired: bool,
}

pub fn event_to_bytes(event: &Event) -> Result<Vec<u8>, bincode::Error> {
    let ser = EventSer {
        id: event.id,
        timestamp: event.timestamp,
        data: event.data.iter().map(AttrSer::from).collect(),
        is_expired: event.is_expired,
    };
    bincode::serialize(&ser)
}

pub fn event_from_bytes(bytes: &[u8]) -> Result<Event, bincode::Error> {
    let ser: EventSer = bincode::deserialize(bytes)?;
    Ok(Event {
        id: ser.id,
        timestamp: ser.timestamp,
        data: ser.data.into_iter().map(AttributeValue::from).collect(),
        is_expired: ser.is_expired,
    })
}
