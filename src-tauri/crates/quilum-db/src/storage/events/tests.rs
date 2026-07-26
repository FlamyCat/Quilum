use crate::Storage;
use chrono::NaiveDate;

#[tokio::test]
async fn event_crud() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let start_time = NaiveDate::from_ymd_opt(2026, 5, 1)
        .unwrap()
        .and_hms_opt(10, 0, 0)
        .unwrap();
    let end_time = NaiveDate::from_ymd_opt(2026, 5, 1)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();

    let event = storage
        .create_event(
            "Test Event".to_string(),
            "A test event".to_string(),
            start_time.and_utc(),
            end_time.and_utc(),
        )
        .await
        .expect("Failed to create event");

    assert!(
        !format!("{}", event.id().table).is_empty(),
        "Event ID should be set"
    );

    let read_event = storage
        .read_event(&event.id())
        .await
        .expect("Failed to read event");
    assert_eq!(read_event.name(), "Test Event");

    let mut updated_event = read_event;
    updated_event.set_name("Updated Event".to_string());
    storage
        .update_event(updated_event)
        .await
        .expect("Failed to update event");

    let updated_read_event = storage
        .read_event(&event.id())
        .await
        .expect("Failed to read updated event");
    assert_eq!(updated_read_event.name(), "Updated Event");

    storage
        .delete_event(&event.id())
        .await
        .expect("Failed to delete event");

    let result = storage.read_event(&event.id()).await;
    assert!(result.is_err());
}
