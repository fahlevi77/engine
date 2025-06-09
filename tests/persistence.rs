use siddhi_rust::core::persistence::{InMemoryPersistenceStore, SnapshotService, PersistenceStore};
use siddhi_rust::core::event::{Event, value::AttributeValue};
use siddhi_rust::core::util::{event_to_bytes, event_from_bytes};
use std::sync::Arc;

#[test]
fn test_event_serialization_roundtrip() {
    let e = Event {
        id: 1,
        timestamp: 123,
        data: vec![AttributeValue::Int(5), AttributeValue::String("hi".into())],
        is_expired: false,
    };
    let bytes = event_to_bytes(&e).unwrap();
    let de = event_from_bytes(&bytes).unwrap();
    assert_eq!(e.id, de.id);
    assert_eq!(e.timestamp, de.timestamp);
    assert_eq!(e.data, de.data);
    assert_eq!(e.is_expired, de.is_expired);
}

#[test]
fn test_snapshot_service_persist_restore() {
    let store = Arc::new(InMemoryPersistenceStore::new());
    let mut service = SnapshotService::new("app1".to_string());
    service.persistence_store = Some(store.clone());
    service.set_state(b"state1".to_vec());
    let rev = service.persist().unwrap();
    service.set_state(b"changed".to_vec());
    service.restore_revision(&rev).unwrap();
    assert_eq!(service.snapshot(), b"state1".to_vec());
    // ensure store recorded revision
    assert_eq!(store.get_last_revision("app1").unwrap(), rev);
}
