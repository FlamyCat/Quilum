use super::*;
use chrono::NaiveDate;

#[tokio::test]
async fn test_storage_mem_creation() {
    let _storage = Storage::new_mem().await.expect("Failed to create storage");
}

#[tokio::test]
async fn test_event_crud() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let start_time = NaiveDate::from_ymd_opt(2026, 5, 1)
        .unwrap()
        .and_hms_opt(10, 0, 0)
        .unwrap();
    let end_time = NaiveDate::from_ymd_opt(2026, 5, 1)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();

    let event = Event::new(
        "Test Event".to_string(),
        "A test event".to_string(),
        start_time,
        end_time,
    );

    let record = storage
        .create_event(event)
        .await
        .expect("Failed to create event");
    let key_is_set =
        matches!(&record.id.key, surrealdb::types::RecordIdKey::String(s) if !s.is_empty());
    assert!(key_is_set, "Record ID key should be set");

    let read_record = storage
        .read_event(&record.id)
        .await
        .expect("Failed to read event");
    assert_eq!(read_record.data.name(), "Test Event");

    let mut updated_event = read_record.data;
    updated_event.set_name("Updated Event".to_string());
    let updated_record = Record {
        id: read_record.id.clone(),
        data: updated_event,
    };
    storage
        .update_event(updated_record)
        .await
        .expect("Failed to update event");

    let updated_read_record = storage
        .read_event(&record.id)
        .await
        .expect("Failed to read updated event");
    assert_eq!(updated_read_record.data.name(), "Updated Event");

    storage
        .delete_event(&record.id)
        .await
        .expect("Failed to delete event");

    let result = storage.read_event(&record.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_events_for_date_range_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    // Create events on different dates
    let event_a = Event::new(
        "Event A".to_string(),
        "On May 1".to_string(),
        date.and_hms_opt(10, 0, 0).unwrap(),
        date.and_hms_opt(12, 0, 0).unwrap(),
    );
    let event_b = Event::new(
        "Event B".to_string(),
        "On May 3".to_string(),
        NaiveDate::from_ymd_opt(2026, 5, 3).unwrap().and_hms_opt(10, 0, 0).unwrap(),
        NaiveDate::from_ymd_opt(2026, 5, 3).unwrap().and_hms_opt(12, 0, 0).unwrap(),
    );
    let event_c = Event::new(
        "Event C".to_string(),
        "On May 5".to_string(),
        NaiveDate::from_ymd_opt(2026, 5, 5).unwrap().and_hms_opt(10, 0, 0).unwrap(),
        NaiveDate::from_ymd_opt(2026, 5, 5).unwrap().and_hms_opt(12, 0, 0).unwrap(),
    );

    storage.create_event(event_a).await.expect("Failed to create event A");
    storage.create_event(event_b).await.expect("Failed to create event B");
    storage.create_event(event_c).await.expect("Failed to create event C");

    // Query range: May 2 to May 4 (should only return event B)
    let start = NaiveDate::from_ymd_opt(2026, 5, 2).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 5, 4).unwrap();
    let events = storage.get_events_for_date_range(start, end).await.expect("Failed to query events");

    assert_eq!(events.len(), 1, "Should return exactly 1 event");
    assert_eq!(events[0].data.name(), "Event B");
}

#[tokio::test]
async fn test_get_events_for_date_range_overlapping() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create event that spans multiple days: May 1 20:00 to May 3 06:00
    let event = Event::new(
        "Multi-day Event".to_string(),
        "Spans 3 days".to_string(),
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap().and_hms_opt(20, 0, 0).unwrap(),
        NaiveDate::from_ymd_opt(2026, 5, 3).unwrap().and_hms_opt(6, 0, 0).unwrap(),
    );
    storage.create_event(event).await.expect("Failed to create event");

    // Query for May 2 (middle day) - should return the event
    let date = NaiveDate::from_ymd_opt(2026, 5, 2).unwrap();
    let events = storage.get_events_for_date(date).await.expect("Failed to query events");

    assert_eq!(events.len(), 1, "Should return the overlapping event");
    assert_eq!(events[0].data.name(), "Multi-day Event");
}

#[tokio::test]
async fn test_get_events_for_date_single() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let date1 = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let date2 = NaiveDate::from_ymd_opt(2026, 5, 2).unwrap();
    let date3 = NaiveDate::from_ymd_opt(2026, 5, 3).unwrap();

    // Create events on different dates
    let event1 = Event::new(
        "Event 1".to_string(),
        "On May 1".to_string(),
        date1.and_hms_opt(10, 0, 0).unwrap(),
        date1.and_hms_opt(12, 0, 0).unwrap(),
    );
    let event2 = Event::new(
        "Event 2".to_string(),
        "On May 2".to_string(),
        date2.and_hms_opt(10, 0, 0).unwrap(),
        date2.and_hms_opt(12, 0, 0).unwrap(),
    );
    let event3 = Event::new(
        "Event 3".to_string(),
        "On May 3".to_string(),
        date3.and_hms_opt(10, 0, 0).unwrap(),
        date3.and_hms_opt(12, 0, 0).unwrap(),
    );

    storage.create_event(event1).await.expect("Failed to create event 1");
    storage.create_event(event2).await.expect("Failed to create event 2");
    storage.create_event(event3).await.expect("Failed to create event 3");

    // Query for May 2 only
    let events = storage.get_events_for_date(date2).await.expect("Failed to query events");

    assert_eq!(events.len(), 1, "Should return exactly 1 event");
    assert_eq!(events[0].data.name(), "Event 2");
}

#[tokio::test]
async fn test_get_events_for_date_range_empty() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Query range with no events
    let start = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
    let events = storage.get_events_for_date_range(start, end).await.expect("Failed to query events");

    assert!(events.is_empty(), "Should return empty vec");
}

#[tokio::test]
async fn test_get_events_for_date_multiple_same_day() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    // Create 3 events on the same day
    for i in 1..=3 {
        let event = Event::new(
            format!("Event {}", i),
            format!("On May 1, event {}", i),
            date.and_hms_opt(i as u32 * 2, 0, 0).unwrap(),
            date.and_hms_opt(i as u32 * 2 + 1, 0, 0).unwrap(),
        );
        storage.create_event(event).await.expect("Failed to create event");
    }

    // Query for May 1
    let events = storage.get_events_for_date(date).await.expect("Failed to query events");

    assert_eq!(events.len(), 3, "Should return all 3 events");
}
