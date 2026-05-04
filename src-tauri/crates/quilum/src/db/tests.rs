use super::*;
use chrono::NaiveDate;

#[tokio::test]
async fn storage_mem_creation() {
    let _storage = Storage::new_mem().await.expect("Failed to create storage");
}

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
async fn get_events_for_date_range_basic() {
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
async fn get_events_for_date_range_overlapping() {
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
async fn get_events_for_date_single() {
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
async fn get_events_for_date_range_empty() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Query range with no events
    let start = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
    let events = storage.get_events_for_date_range(start, end).await.expect("Failed to query events");

    assert!(events.is_empty(), "Should return empty vec");
}

#[tokio::test]
async fn get_events_for_date_multiple_same_day() {
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

#[tokio::test]
async fn get_scheduled_tasks_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a slot on 2026-05-01
    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = Slot::new(
        slot_date.and_hms_opt(10, 0, 0).unwrap(),
        slot_date.and_hms_opt(12, 0, 0).unwrap(),
    );
    let slot_record = storage.create_slot(slot).await.expect("Failed to create slot");

    // Create a task
    let task = Task::new(
        "Test Task".to_string(),
        "A scheduled task".to_string(),
        crate::model::task::Priority::Medium,
        chrono::TimeDelta::hours(1),
        slot_date.and_hms_opt(0, 0, 0).unwrap(),
    );
    let task_record = storage.create_task(task).await.expect("Failed to create task");

    // Relate task to slot with scheduled_for
    let scheduled_for = slot_date.and_hms_opt(10, 30, 0).unwrap();
    storage.relate_task_to_slot(&slot_record.id, &task_record.id, scheduled_for)
        .await
        .expect("Failed to relate task to slot");

    // Query for May 1
    let scheduled_tasks = storage
        .get_scheduled_tasks_for_date_range(slot_date, slot_date + chrono::TimeDelta::days(1))
        .await
        .expect("Failed to query scheduled tasks");

    assert_eq!(scheduled_tasks.len(), 1, "Should return 1 scheduled task");
    assert_eq!(scheduled_tasks[0].task.data.name(), "Test Task");
    assert_eq!(scheduled_tasks[0].scheduled_for, scheduled_for.and_utc().timestamp());
}

#[tokio::test]
async fn get_scheduled_tasks_wrong_date() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a slot on 2026-05-01
    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = Slot::new(
        slot_date.and_hms_opt(10, 0, 0).unwrap(),
        slot_date.and_hms_opt(12, 0, 0).unwrap(),
    );
    let slot_record = storage.create_slot(slot).await.expect("Failed to create slot");

    // Create a task and relate to slot
    let task = Task::new(
        "Test Task".to_string(),
        "A scheduled task".to_string(),
        crate::model::task::Priority::Medium,
        chrono::TimeDelta::hours(1),
        slot_date.and_hms_opt(0, 0, 0).unwrap(),
    );
    let task_record = storage.create_task(task).await.expect("Failed to create task");

    storage.relate_task_to_slot(&slot_record.id, &task_record.id, slot_date.and_hms_opt(10, 0, 0).unwrap())
        .await
        .expect("Failed to relate task to slot");

    // Query for May 2 (wrong date)
    let wrong_date = NaiveDate::from_ymd_opt(2026, 5, 2).unwrap();
    let scheduled_tasks = storage
        .get_scheduled_tasks_for_date_range(wrong_date, wrong_date + chrono::TimeDelta::days(1))
        .await
        .expect("Failed to query scheduled tasks");

    assert!(scheduled_tasks.is_empty(), "Should return empty vec for wrong date");
}

#[tokio::test]
async fn get_scheduled_tasks_multiple_in_slot() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a slot on 2026-05-01
    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = Slot::new(
        slot_date.and_hms_opt(10, 0, 0).unwrap(),
        slot_date.and_hms_opt(14, 0, 0).unwrap(),
    );
    let slot_record = storage.create_slot(slot).await.expect("Failed to create slot");

    // Create 3 tasks and relate to slot
    let mut expected_scheduled_fors = Vec::new();
    for i in 1..=3 {
        let task = Task::new(
            format!("Task {}", i),
            format!("Scheduled task {}", i),
            crate::model::task::Priority::Medium,
            chrono::TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap(),
        );
        let task_record = storage.create_task(task).await.expect("Failed to create task");

        let scheduled_for = slot_date.and_hms_opt(10 + (i - 1) as u32, 0, 0).unwrap();
        storage.relate_task_to_slot(&slot_record.id, &task_record.id, scheduled_for)
            .await
            .expect("Failed to relate task to slot");
        expected_scheduled_fors.push(scheduled_for.and_utc().timestamp());
    }

    // Query for May 1
    let scheduled_tasks = storage
        .get_scheduled_tasks_for_date_range(slot_date, slot_date + chrono::TimeDelta::days(1))
        .await
        .expect("Failed to query scheduled tasks");

    assert_eq!(scheduled_tasks.len(), 3, "Should return all 3 tasks");

    // Verify all task names and scheduled_for times
    let mut task_names: Vec<&str> = scheduled_tasks.iter().map(|st| st.task.data.name()).collect();
    task_names.sort();
    assert_eq!(task_names, vec!["Task 1", "Task 2", "Task 3"]);

    let mut scheduled_fors: Vec<i64> = scheduled_tasks.iter().map(|st| st.scheduled_for).collect();
    scheduled_fors.sort();
    assert_eq!(scheduled_fors, expected_scheduled_fors);
}

#[tokio::test]
async fn get_scheduled_tasks_date_range_filter() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Slot A on 2026-05-01 with task T1
    let date1 = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot_a = Slot::new(
        date1.and_hms_opt(10, 0, 0).unwrap(),
        date1.and_hms_opt(12, 0, 0).unwrap(),
    );
    let slot_a_record = storage.create_slot(slot_a).await.expect("Failed to create slot A");

    let task_t1 = Task::new(
        "Task T1".to_string(),
        "In slot A".to_string(),
        crate::model::task::Priority::Medium,
        chrono::TimeDelta::hours(1),
        date1.and_hms_opt(0, 0, 0).unwrap(),
    );
    let task_t1_record = storage.create_task(task_t1).await.expect("Failed to create T1");
    storage.relate_task_to_slot(&slot_a_record.id, &task_t1_record.id, date1.and_hms_opt(10, 0, 0).unwrap())
        .await
        .expect("Failed to relate T1 to slot A");

    // Slot B on 2026-05-03 with task T2
    let date2 = NaiveDate::from_ymd_opt(2026, 5, 3).unwrap();
    let slot_b = Slot::new(
        date2.and_hms_opt(10, 0, 0).unwrap(),
        date2.and_hms_opt(12, 0, 0).unwrap(),
    );
    let slot_b_record = storage.create_slot(slot_b).await.expect("Failed to create slot B");

    let task_t2 = Task::new(
        "Task T2".to_string(),
        "In slot B".to_string(),
        crate::model::task::Priority::Medium,
        chrono::TimeDelta::hours(1),
        date2.and_hms_opt(0, 0, 0).unwrap(),
    );
    let task_t2_record = storage.create_task(task_t2).await.expect("Failed to create T2");
    storage.relate_task_to_slot(&slot_b_record.id, &task_t2_record.id, date2.and_hms_opt(10, 0, 0).unwrap())
        .await
        .expect("Failed to relate T2 to slot B");

    // Query range: 2026-05-01 to 2026-05-03 (exclusive end, so May 3 is excluded)
    let scheduled_tasks = storage
        .get_scheduled_tasks_for_date_range(date1, date2)
        .await
        .expect("Failed to query scheduled tasks");

    assert_eq!(scheduled_tasks.len(), 1, "Should return only T1 (May 3 is excluded)");
    assert_eq!(scheduled_tasks[0].task.data.name(), "Task T1");
}

#[tokio::test]
async fn get_scheduled_tasks_empty_result() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Query range with no slots/tasks
    let start = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
    let scheduled_tasks = storage
        .get_scheduled_tasks_for_date_range(start, end)
        .await
        .expect("Failed to query scheduled tasks");

    assert!(scheduled_tasks.is_empty(), "Should return empty vec when no data");
}
