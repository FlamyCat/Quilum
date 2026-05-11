use super::*;
use crate::app_identifier::AppIdentifier;
use chrono::NaiveDate;
use std::path::PathBuf;
use surrealdb::types::RecordId;

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

    let event = storage
        .create_event(
            "Test Event".to_string(),
            "A test event".to_string(),
            start_time,
            end_time,
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

#[tokio::test]
async fn get_events_for_date_range_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    // Create events on different dates
    let _event_a = storage
        .create_event(
            "Event A".to_string(),
            "On May 1".to_string(),
            date.and_hms_opt(10, 0, 0).unwrap(),
            date.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create event A");

    let _event_b = storage
        .create_event(
            "Event B".to_string(),
            "On May 3".to_string(),
            NaiveDate::from_ymd_opt(2026, 5, 3)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap(),
            NaiveDate::from_ymd_opt(2026, 5, 3)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap(),
        )
        .await
        .expect("Failed to create event B");

    let _event_c = storage
        .create_event(
            "Event C".to_string(),
            "On May 5".to_string(),
            NaiveDate::from_ymd_opt(2026, 5, 5)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap(),
            NaiveDate::from_ymd_opt(2026, 5, 5)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap(),
        )
        .await
        .expect("Failed to create event C");

    // Query range: May 2 to May 4 (should only return event B)
    let start = NaiveDate::from_ymd_opt(2026, 5, 2).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 5, 4).unwrap();
    let events = storage
        .get_events_for_date_range(start, end)
        .await
        .expect("Failed to query events");

    assert_eq!(events.len(), 1, "Should return exactly 1 event");
    assert_eq!(events[0].name(), "Event B");
}

#[tokio::test]
async fn get_events_for_date_range_overlapping() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create event that spans multiple days: May 1 20:00 to May 3 06:00
    let _event = storage
        .create_event(
            "Multi-day Event".to_string(),
            "Spans 3 days".to_string(),
            NaiveDate::from_ymd_opt(2026, 5, 1)
                .unwrap()
                .and_hms_opt(20, 0, 0)
                .unwrap(),
            NaiveDate::from_ymd_opt(2026, 5, 3)
                .unwrap()
                .and_hms_opt(6, 0, 0)
                .unwrap(),
        )
        .await
        .expect("Failed to create event");

    // Query for May 2 (middle day) - should return the event
    let date = NaiveDate::from_ymd_opt(2026, 5, 2).unwrap();
    let events = storage
        .get_events_for_date(date)
        .await
        .expect("Failed to query events");

    assert_eq!(events.len(), 1, "Should return the overlapping event");
    assert_eq!(events[0].name(), "Multi-day Event");
}

#[tokio::test]
async fn get_events_for_date_single() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let date1 = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let date2 = NaiveDate::from_ymd_opt(2026, 5, 2).unwrap();
    let date3 = NaiveDate::from_ymd_opt(2026, 5, 3).unwrap();

    // Create events on different dates
    storage
        .create_event(
            "Event 1".to_string(),
            "On May 1".to_string(),
            date1.and_hms_opt(10, 0, 0).unwrap(),
            date1.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create event 1");

    storage
        .create_event(
            "Event 2".to_string(),
            "On May 2".to_string(),
            date2.and_hms_opt(10, 0, 0).unwrap(),
            date2.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create event 2");

    storage
        .create_event(
            "Event 3".to_string(),
            "On May 3".to_string(),
            date3.and_hms_opt(10, 0, 0).unwrap(),
            date3.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create event 3");

    // Query for May 2 only
    let events = storage
        .get_events_for_date(date2)
        .await
        .expect("Failed to query events");

    assert_eq!(events.len(), 1, "Should return exactly 1 event");
    assert_eq!(events[0].name(), "Event 2");
}

#[tokio::test]
async fn get_events_for_date_range_empty() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Query range with no events
    let start = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
    let events = storage
        .get_events_for_date_range(start, end)
        .await
        .expect("Failed to query events");

    assert!(events.is_empty(), "Should return empty vec");
}

#[tokio::test]
async fn get_events_for_date_multiple_same_day() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    // Create 3 events on the same day
    for i in 1..=3 {
        storage
            .create_event(
                format!("Event {}", i),
                format!("On May 1, event {}", i),
                date.and_hms_opt(i as u32 * 2, 0, 0).unwrap(),
                date.and_hms_opt(i as u32 * 2 + 1, 0, 0).unwrap(),
            )
            .await
            .expect("Failed to create event");
    }

    // Query for May 1
    let events = storage
        .get_events_for_date(date)
        .await
        .expect("Failed to query events");

    assert_eq!(events.len(), 3, "Should return all 3 events");
}

#[tokio::test]
async fn get_scheduled_tasks_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a slot on 2026-05-01
    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap(),
            slot_date.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot");

    // Create a task
    let task = storage
        .create_task(
            "Test Task".to_string(),
            "A scheduled task".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create task");
    assert_eq!(task.completed, false);

    // Relate task to slot with scheduled_for
    let scheduled_for = slot_date.and_hms_opt(10, 30, 0).unwrap();
    storage
        .relate_task_to_slot(&slot.id(), &task.id(), scheduled_for)
        .await
        .expect("Failed to relate task to slot");

    // Query for May 1
    let scheduled_tasks = storage
        .get_scheduled_tasks_for_date_range(slot_date, slot_date + TimeDelta::days(1))
        .await
        .expect("Failed to query scheduled tasks");

    assert_eq!(scheduled_tasks.len(), 1, "Should return 1 scheduled task");
    assert_eq!(scheduled_tasks[0].0.name(), "Test Task");
    assert_eq!(scheduled_tasks[0].1, scheduled_for.and_utc().timestamp());
}

#[tokio::test]
async fn get_scheduled_tasks_wrong_date() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a slot on 2026-05-01
    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap(),
            slot_date.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot");

    // Create a task and relate to slot
    let task = storage
        .create_task(
            "Test Task".to_string(),
            "A scheduled task".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create task");
    assert_eq!(task.completed, false);

    storage
        .relate_task_to_slot(
            &slot.id(),
            &task.id(),
            slot_date.and_hms_opt(10, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to relate task to slot");

    // Query for May 2 (wrong date)
    let wrong_date = NaiveDate::from_ymd_opt(2026, 5, 2).unwrap();
    let scheduled_tasks = storage
        .get_scheduled_tasks_for_date_range(wrong_date, wrong_date + TimeDelta::days(1))
        .await
        .expect("Failed to query scheduled tasks");

    assert!(
        scheduled_tasks.is_empty(),
        "Should return empty vec for wrong date"
    );
}

#[tokio::test]
async fn get_scheduled_tasks_multiple_in_slot() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a slot on 2026-05-01
    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap(),
            slot_date.and_hms_opt(14, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot");

    // Create 3 tasks and relate to slot
    let mut expected_scheduled_fors = Vec::new();
    for i in 1..=3 {
        let task = storage
            .create_task(
                format!("Task {}", i),
                format!("Scheduled task {}", i),
                Priority::Medium,
                TimeDelta::hours(1),
                slot_date.and_hms_opt(0, 0, 0).unwrap(),
            )
            .await
            .expect("Failed to create task");
        assert_eq!(task.completed, false);

        let scheduled_for = slot_date.and_hms_opt(10 + (i - 1) as u32, 0, 0).unwrap();
        storage
            .relate_task_to_slot(&slot.id(), &task.id(), scheduled_for)
            .await
            .expect("Failed to relate task to slot");
        expected_scheduled_fors.push(scheduled_for.and_utc().timestamp());
    }

    // Query for May 1
    let scheduled_tasks = storage
        .get_scheduled_tasks_for_date_range(slot_date, slot_date + TimeDelta::days(1))
        .await
        .expect("Failed to query scheduled tasks");

    assert_eq!(scheduled_tasks.len(), 3, "Should return all 3 tasks");

    // Verify all task names and scheduled_for times
    let mut task_names: Vec<&str> = scheduled_tasks.iter().map(|st| st.0.name()).collect();
    task_names.sort();
    assert_eq!(task_names, vec!["Task 1", "Task 2", "Task 3"]);

    let mut scheduled_fors: Vec<i64> = scheduled_tasks.iter().map(|st| st.1).collect();
    scheduled_fors.sort();
    assert_eq!(scheduled_fors, expected_scheduled_fors);
}

#[tokio::test]
async fn get_scheduled_tasks_date_range_filter() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Slot A on 2026-05-01 with task T1
    let date1 = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot_a = storage
        .create_slot(
            date1.and_hms_opt(10, 0, 0).unwrap(),
            date1.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot A");

    let task_t1 = storage
        .create_task(
            "Task T1".to_string(),
            "In slot A".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            date1.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create T1");
    assert_eq!(task_t1.completed, false);
    storage
        .relate_task_to_slot(
            &slot_a.id(),
            &task_t1.id(),
            date1.and_hms_opt(10, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to relate T1 to slot A");

    // Slot B on 2026-05-03 with task T2
    let date2 = NaiveDate::from_ymd_opt(2026, 5, 3).unwrap();
    let slot_b = storage
        .create_slot(
            date2.and_hms_opt(10, 0, 0).unwrap(),
            date2.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot B");

    let task_t2 = storage
        .create_task(
            "Task T2".to_string(),
            "In slot B".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            date2.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create T2");
    assert_eq!(task_t2.completed, false);
    storage
        .relate_task_to_slot(
            &slot_b.id(),
            &task_t2.id(),
            date2.and_hms_opt(10, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to relate T2 to slot B");

    // Query range: 2026-05-01 to 2026-05-03 (exclusive end, so May 3 is excluded)
    let scheduled_tasks = storage
        .get_scheduled_tasks_for_date_range(date1, date2)
        .await
        .expect("Failed to query scheduled tasks");

    assert_eq!(
        scheduled_tasks.len(),
        1,
        "Should return only T1 (May 3 is excluded)"
    );
    assert_eq!(scheduled_tasks[0].0.name(), "Task T1");
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

    assert!(
        scheduled_tasks.is_empty(),
        "Should return empty vec when no data"
    );
}

#[tokio::test]
async fn get_slots_with_tasks_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a slot on 2026-05-01
    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap(),
            slot_date.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot");

    // Create a task
    let task = storage
        .create_task(
            "Test Task".to_string(),
            "A scheduled task".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create task");
    assert_eq!(task.completed, false);

    // Relate task to slot with scheduled_for
    let scheduled_for = slot_date.and_hms_opt(10, 30, 0).unwrap();
    storage
        .relate_task_to_slot(&slot.id(), &task.id(), scheduled_for)
        .await
        .expect("Failed to relate task to slot");

    // Query for May 1
    let slots_with_tasks = storage
        .get_slots_with_tasks_for_date_range(slot_date, slot_date + TimeDelta::days(1))
        .await
        .expect("Failed to query slots with tasks");

    assert_eq!(slots_with_tasks.len(), 1, "Should return 1 slot");
    assert_eq!(
        slots_with_tasks[0].slot.starts_at(),
        slot_date.and_hms_opt(10, 0, 0).unwrap()
    );
    assert_eq!(slots_with_tasks[0].tasks.len(), 1, "Should have 1 task");
    assert_eq!(slots_with_tasks[0].tasks[0].0.name(), "Test Task");
    assert_eq!(
        slots_with_tasks[0].tasks[0].1,
        scheduled_for.and_utc().timestamp()
    );
}

#[tokio::test]
async fn get_slots_with_tasks_multiple_tasks() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a slot on 2026-05-01
    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap(),
            slot_date.and_hms_opt(14, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot");

    // Create 3 tasks and relate to slot
    let mut expected_scheduled_fors = Vec::new();
    for i in 1..=3 {
        let task = storage
            .create_task(
                format!("Task {}", i),
                format!("Scheduled task {}", i),
                Priority::Medium,
                TimeDelta::hours(1),
                slot_date.and_hms_opt(0, 0, 0).unwrap(),
            )
            .await
            .expect("Failed to create task");
        assert_eq!(task.completed, false);

        let scheduled_for = slot_date.and_hms_opt(10 + (i - 1) as u32, 0, 0).unwrap();
        storage
            .relate_task_to_slot(&slot.id(), &task.id(), scheduled_for)
            .await
            .expect("Failed to relate task to slot");
        expected_scheduled_fors.push(scheduled_for.and_utc().timestamp());
    }

    // Query for May 1
    let slots_with_tasks = storage
        .get_slots_with_tasks_for_date_range(slot_date, slot_date + TimeDelta::days(1))
        .await
        .expect("Failed to query slots with tasks");

    assert_eq!(slots_with_tasks.len(), 1, "Should return 1 slot");
    assert_eq!(slots_with_tasks[0].tasks.len(), 3, "Should have 3 tasks");

    // Verify all task names and scheduled_for times
    let mut task_names: Vec<&str> = slots_with_tasks[0]
        .tasks
        .iter()
        .map(|(t, _)| t.name())
        .collect();
    task_names.sort();
    assert_eq!(task_names, vec!["Task 1", "Task 2", "Task 3"]);

    let mut scheduled_fors: Vec<i64> = slots_with_tasks[0]
        .tasks
        .iter()
        .map(|(_, sf)| *sf)
        .collect();
    scheduled_fors.sort();
    assert_eq!(scheduled_fors, expected_scheduled_fors);
}

#[tokio::test]
async fn get_slots_with_tasks_multiple_slots() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    // Slot A with 2 tasks
    let slot_a = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap(),
            slot_date.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot A");

    for i in 1..=2 {
        let task = storage
            .create_task(
                format!("Task A{}", i),
                format!("In slot A"),
                Priority::Medium,
                TimeDelta::hours(1),
                slot_date.and_hms_opt(0, 0, 0).unwrap(),
            )
            .await
            .expect("Failed to create task");
        assert_eq!(task.completed, false);

        storage
            .relate_task_to_slot(
                &slot_a.id(),
                &task.id(),
                slot_date.and_hms_opt(10 + (i - 1) as u32, 0, 0).unwrap(),
            )
            .await
            .expect("Failed to relate task to slot A");
    }

    // Slot B with 1 task
    let slot_b = storage
        .create_slot(
            slot_date.and_hms_opt(14, 0, 0).unwrap(),
            slot_date.and_hms_opt(16, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot B");

    let task_b = storage
        .create_task(
            "Task B1".to_string(),
            "In slot B".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create task");
    assert_eq!(task_b.completed, false);

    storage
        .relate_task_to_slot(
            &slot_b.id(),
            &task_b.id(),
            slot_date.and_hms_opt(14, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to relate task to slot B");

    // Query for May 1
    let slots_with_tasks = storage
        .get_slots_with_tasks_for_date_range(slot_date, slot_date + TimeDelta::days(1))
        .await
        .expect("Failed to query slots with tasks");

    assert_eq!(slots_with_tasks.len(), 2, "Should return 2 slots");

    // Find Slot A and Slot B
    let slot_a_result = slots_with_tasks
        .iter()
        .find(|s| s.slot.id() == slot_a.id())
        .expect("Slot A not found");
    let slot_b_result = slots_with_tasks
        .iter()
        .find(|s| s.slot.id() == slot_b.id())
        .expect("Slot B not found");

    assert_eq!(slot_a_result.tasks.len(), 2, "Slot A should have 2 tasks");
    assert_eq!(slot_b_result.tasks.len(), 1, "Slot B should have 1 task");
}

#[tokio::test]
async fn get_slots_with_tasks_date_range_filter() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Slot A on 2026-05-01
    let date1 = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot_a = storage
        .create_slot(
            date1.and_hms_opt(10, 0, 0).unwrap(),
            date1.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot A");

    let task_t1 = storage
        .create_task(
            "Task T1".to_string(),
            "In slot A".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            date1.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create T1");
    storage
        .relate_task_to_slot(
            &slot_a.id(),
            &task_t1.id(),
            date1.and_hms_opt(10, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to relate T1 to slot A");

    // Slot B on 2026-05-03
    let date2 = NaiveDate::from_ymd_opt(2026, 5, 3).unwrap();
    let slot_b = storage
        .create_slot(
            date2.and_hms_opt(10, 0, 0).unwrap(),
            date2.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot B");

    let task_t2 = storage
        .create_task(
            "Task T2".to_string(),
            "In slot B".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            date2.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create T2");
    storage
        .relate_task_to_slot(
            &slot_b.id(),
            &task_t2.id(),
            date2.and_hms_opt(10, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to relate T2 to slot B");

    // Query range: 2026-05-01 to 2026-05-03 (exclusive end, so May 3 is excluded)
    let slots_with_tasks = storage
        .get_slots_with_tasks_for_date_range(date1, date2)
        .await
        .expect("Failed to query slots with tasks");

    assert_eq!(
        slots_with_tasks.len(),
        1,
        "Should return only Slot A (May 3 is excluded)"
    );
    assert_eq!(slots_with_tasks[0].slot.id(), slot_a.id());
    assert_eq!(slots_with_tasks[0].tasks[0].0.name(), "Task T1");
}

#[tokio::test]
async fn get_slots_with_tasks_empty_result() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Query range with no slots
    let start = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
    let slots_with_tasks = storage
        .get_slots_with_tasks_for_date_range(start, end)
        .await
        .expect("Failed to query slots with tasks");

    assert!(
        slots_with_tasks.is_empty(),
        "Should return empty vec when no slots"
    );
}

#[tokio::test]
async fn get_slots_with_tasks_slot_without_tasks() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a slot with NO tasks
    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let _slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap(),
            slot_date.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot");

    // Query for May 1
    let slots_with_tasks = storage
        .get_slots_with_tasks_for_date_range(slot_date, slot_date + TimeDelta::days(1))
        .await
        .expect("Failed to query slots with tasks");

    assert_eq!(slots_with_tasks.len(), 1, "Should return 1 slot");
    assert_eq!(
        slots_with_tasks[0].tasks.len(),
        0,
        "Slot should have no tasks"
    );
}

#[tokio::test]
async fn get_today_timetable_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    // Create an event for today
    let _event = storage
        .create_event(
            "Today's Event".to_string(),
            "An event today".to_string(),
            today.and_hms_opt(9, 0, 0).unwrap(),
            today.and_hms_opt(10, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create event");

    // Create a slot with a task for today
    let slot = storage
        .create_slot(
            today.and_hms_opt(10, 0, 0).unwrap(),
            today.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot");

    let task = storage
        .create_task(
            "Today's Task".to_string(),
            "A task for today".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            today.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create task");
    assert_eq!(task.completed, false);

    let scheduled_for = today.and_hms_opt(10, 30, 0).unwrap();
    storage
        .relate_task_to_slot(&slot.id(), &task.id(), scheduled_for)
        .await
        .expect("Failed to relate task to slot");

    // Get today's timetable
    let (events, scheduled_tasks) = storage
        .get_today_timetable(today)
        .await
        .expect("Failed to get today timetable");

    assert_eq!(events.len(), 1, "Should have 1 event");
    assert_eq!(events[0].name(), "Today's Event");
    assert_eq!(scheduled_tasks.len(), 1, "Should have 1 scheduled task");
    assert_eq!(scheduled_tasks[0].0.name(), "Today's Task");
    assert_eq!(scheduled_tasks[0].1, scheduled_for.and_utc().timestamp());
}

#[tokio::test]
async fn get_today_timetable_empty() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");
    let today = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();

    // Get today's timetable with no data
    let (events, scheduled_tasks) = storage
        .get_today_timetable(today)
        .await
        .expect("Failed to get today timetable");

    assert!(events.is_empty(), "Events should be empty");
    assert!(
        scheduled_tasks.is_empty(),
        "Scheduled tasks should be empty"
    );
}

#[tokio::test]
async fn get_week_timetable_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");
    let week_start = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    // Event A on May 1
    let _event_a = storage
        .create_event(
            "Event A".to_string(),
            "On May 1".to_string(),
            week_start.and_hms_opt(10, 0, 0).unwrap(),
            week_start.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create event A");

    // Slot B on May 2 with 2 tasks
    let slot_b = storage
        .create_slot(
            week_start.and_hms_opt(14, 0, 0).unwrap(),
            week_start.and_hms_opt(18, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot B");

    for i in 1..=2 {
        let task = storage
            .create_task(
                format!("Task B{}", i),
                format!("In slot B"),
                Priority::Medium,
                TimeDelta::hours(1),
                week_start.and_hms_opt(0, 0, 0).unwrap(),
            )
            .await
            .expect("Failed to create task");
        assert_eq!(task.completed, false);

        storage
            .relate_task_to_slot(
                &slot_b.id(),
                &task.id(),
                week_start.and_hms_opt(14 + (i - 1) as u32, 0, 0).unwrap(),
            )
            .await
            .expect("Failed to relate task to slot B");
    }

    // Event C on May 5
    let _event_c = storage
        .create_event(
            "Event C".to_string(),
            "On May 5".to_string(),
            NaiveDate::from_ymd_opt(2026, 5, 5)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap(),
            NaiveDate::from_ymd_opt(2026, 5, 5)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap(),
        )
        .await
        .expect("Failed to create event C");

    // Get week timetable (May 1 to May 8, exclusive end)
    let (events, slots_with_tasks) = storage
        .get_week_timetable(week_start)
        .await
        .expect("Failed to get week timetable");

    assert_eq!(events.len(), 2, "Should have 2 events (A and C)");
    assert_eq!(slots_with_tasks.len(), 1, "Should have 1 slot (B)");
    assert_eq!(
        slots_with_tasks[0].tasks.len(),
        2,
        "Slot B should have 2 tasks"
    );
}

#[tokio::test]
async fn get_week_timetable_excludes_next_week() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");
    let week_start = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    // Slot A on May 1 (within week)
    let _slot_a = storage
        .create_slot(
            week_start.and_hms_opt(10, 0, 0).unwrap(),
            week_start.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot A");

    // Slot B on May 8 (next week, should be excluded)
    let slot_b_date = NaiveDate::from_ymd_opt(2026, 5, 8).unwrap();
    let slot_b = storage
        .create_slot(
            slot_b_date.and_hms_opt(10, 0, 0).unwrap(),
            slot_b_date.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot B");

    let task_b = storage
        .create_task(
            "Task B".to_string(),
            "In slot B (next week)".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_b_date.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create task");
    assert_eq!(task_b.completed, false);

    storage
        .relate_task_to_slot(
            &slot_b.id(),
            &task_b.id(),
            slot_b_date.and_hms_opt(10, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to relate task to slot B");

    // Get week timetable (May 1 to May 8, exclusive end)
    let (_, slots_with_tasks) = storage
        .get_week_timetable(week_start)
        .await
        .expect("Failed to get week timetable");

    assert_eq!(slots_with_tasks.len(), 1, "Should have only 1 slot (A)");
    assert_eq!(
        slots_with_tasks[0].slot.id(),
        _slot_a.id(),
        "Should be slot A"
    );
}

#[tokio::test]
async fn task_list_crud() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a task list
    let task_list = storage
        .create_task_list("My Tasks".to_string())
        .await
        .expect("Failed to create task list");

    assert_eq!(task_list.title, "My Tasks");
    assert!(!format!("{}", task_list.id().table).is_empty());

    // Read the task list
    let read_list = storage
        .read_task_list(task_list.id())
        .await
        .expect("Failed to read task list");
    assert_eq!(read_list.title, "My Tasks");

    // Update the task list
    let mut updated_list = read_list;
    updated_list.title = "Updated Tasks".to_string();
    storage
        .update_task_list(updated_list)
        .await
        .expect("Failed to update task list");

    let updated_read = storage
        .read_task_list(task_list.id())
        .await
        .expect("Failed to read updated task list");
    assert_eq!(updated_read.title, "Updated Tasks");

    // Delete the task list
    storage
        .delete_task_list(task_list.id())
        .await
        .expect("Failed to delete task list");

    let result = storage.read_task_list(task_list.id()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn relate_task_to_list() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a task list
    let task_list = storage
        .create_task_list("Work Tasks".to_string())
        .await
        .expect("Failed to create task list");

    // Create a task
    let task = storage
        .create_task(
            "Implement feature".to_string(),
            "A task to implement".to_string(),
            Priority::High,
            TimeDelta::hours(2),
            NaiveDate::from_ymd_opt(2026, 5, 1)
                .unwrap()
                .and_hms_opt(17, 0, 0)
                .unwrap(),
        )
        .await
        .expect("Failed to create task");

    // Relate task to list
    storage
        .relate_task_to_list(task.id(), task_list.id())
        .await
        .expect("Failed to relate task to list");

    // Get tasks in the list
    let tasks = storage
        .get_tasks_in_list(task_list.id())
        .await
        .expect("Failed to get tasks in list");

    assert_eq!(tasks.len(), 1, "Should have 1 task");
    assert_eq!(tasks[0].name(), "Implement feature");
}

#[tokio::test]
async fn debug_get_tasks_in_list_v2() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let list = storage
        .create_task_list("Test".to_string())
        .await
        .expect("Failed");
    let task = storage
        .create_task(
            "My Task".to_string(),
            "desc".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            NaiveDate::from_ymd_opt(2026, 5, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .await
        .expect("Failed");

    storage
        .relate_task_to_list(task.id(), list.id())
        .await
        .expect("Failed");

    let sql = format!(
        "SELECT out.* FROM belongs_to WHERE out = {}",
        format!(
            "{}:{}",
            list.id().table,
            match &list.id().key {
                surrealdb::types::RecordIdKey::String(s) => s.as_str(),
                _ => "x",
            }
        )
    );
    println!("SQL: {}", sql);

    let mut result = storage.db.query(&sql).await.unwrap();
    let raw: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
    println!("Raw results: {:?}", raw);

    if let Some(first) = raw.first() {
        println!("First item: {:?}", first);
        if let Some(out) = first.get("out") {
            println!("out value: {:?}", out);
            if let Some(obj) = out.as_object() {
                println!("out keys: {:?}", obj.keys().collect::<Vec<_>>());
            }
        }
    }

    let tasks = storage.get_tasks_in_list(list.id()).await.expect("Failed");
    println!("Got {} tasks", tasks.len());
}

#[tokio::test]
async fn debug_get_tasks_in_list() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create list
    let list = storage
        .create_task_list("Test".to_string())
        .await
        .expect("Failed");
    let list_id_str = format!(
        "{}:{}",
        list.id().table,
        match &list.id().key {
            surrealdb::types::RecordIdKey::String(s) => s.as_str(),
            _ => "unknown",
        }
    );
    println!("List ID: {}", list_id_str);

    // Create 2 tasks and relate
    for i in 1..=2 {
        let task = storage
            .create_task(
                format!("Task {}", i),
                "desc".to_string(),
                Priority::Medium,
                TimeDelta::hours(1),
                NaiveDate::from_ymd_opt(2026, 5, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
            )
            .await
            .expect("Failed");

        let task_id_str = format!(
            "{}:{}",
            task.id().table,
            match &task.id().key {
                surrealdb::types::RecordIdKey::String(s) => s.as_str(),
                _ => "unknown",
            }
        );

        storage
            .relate_task_to_list(task.id(), list.id())
            .await
            .expect("Failed");
        println!("Created task {} with id: {}", i, task_id_str);
    }

    // Query belongs_to table manually
    let mut result = storage.db.query("SELECT * FROM belongs_to").await.unwrap();
    let belongs: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
    println!("All belongs_to: {:?}", belongs);

    // Try get_tasks_in_list
    let tasks = storage.get_tasks_in_list(list.id()).await.expect("Failed");
    println!("get_tasks_in_list returned: {}", tasks.len());
}

#[tokio::test]
async fn get_all_task_lists_with_tasks() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create list 1 with 2 tasks
    let list1 = storage
        .create_task_list("List 1".to_string())
        .await
        .expect("Failed to create list 1");

    let task1a = storage
        .create_task(
            "Task 1A".to_string(),
            "First task in list 1".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            NaiveDate::from_ymd_opt(2026, 5, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .await
        .expect("Failed to create task 1A");
    storage
        .relate_task_to_list(task1a.id(), list1.id())
        .await
        .expect("Failed to relate task 1A");

    let task1b = storage
        .create_task(
            "Task 1B".to_string(),
            "Second task in list 1".to_string(),
            Priority::Low,
            TimeDelta::hours(2),
            NaiveDate::from_ymd_opt(2026, 5, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .await
        .expect("Failed to create task 1B");
    storage
        .relate_task_to_list(task1b.id(), list1.id())
        .await
        .expect("Failed to relate task 1B");

    // Create list 2 with 1 task
    let list2 = storage
        .create_task_list("List 2".to_string())
        .await
        .expect("Failed to create list 2");

    let task2a = storage
        .create_task(
            "Task 2A".to_string(),
            "Only task in list 2".to_string(),
            Priority::High,
            TimeDelta::minutes(30),
            NaiveDate::from_ymd_opt(2026, 5, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .await
        .expect("Failed to create task 2A");
    storage
        .relate_task_to_list(task2a.id(), list2.id())
        .await
        .expect("Failed to relate task 2A");

    // Create list 3 with no tasks
    let _list3 = storage
        .create_task_list("Empty List".to_string())
        .await
        .expect("Failed to create empty list");

    // Get all task lists with tasks
    let all_lists = storage
        .get_all_task_lists_with_tasks()
        .await
        .expect("Failed to get all task lists");

    assert_eq!(all_lists.len(), 3, "Should have 3 lists");

    // Find list 1
    let found_list1 = all_lists
        .iter()
        .find(|l| l.list.title() == "List 1")
        .expect("List 1 not found");
    assert_eq!(found_list1.tasks.len(), 2, "List 1 should have 2 tasks");

    // Find list 2
    let found_list2 = all_lists
        .iter()
        .find(|l| l.list.title() == "List 2")
        .expect("List 2 not found");
    assert_eq!(found_list2.tasks.len(), 1, "List 2 should have 1 task");

    // Find list 3
    let found_list3 = all_lists
        .iter()
        .find(|l| l.list.title() == "Empty List")
        .expect("Empty list not found");
    assert_eq!(found_list3.tasks.len(), 0, "Empty list should have 0 tasks");
}

#[tokio::test]
async fn debug_relate_task_to_list() {
    use surrealdb::types::RecordIdKey;

    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a task list
    let task_list = storage
        .create_task_list("Debug List".to_string())
        .await
        .expect("Failed to create task list");
    println!("Created list: {:?}", task_list.id());

    // Create a task
    let task = storage
        .create_task(
            "Debug Task".to_string(),
            "Testing".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            NaiveDate::from_ymd_opt(2026, 5, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .await
        .expect("Failed to create task");
    println!("Created task: {:?}", task.id());

    // Build the exact SQL the function uses
    fn record_id_to_string(id: &RecordId) -> String {
        match &id.key {
            RecordIdKey::String(s) => format!("{}:{}", id.table, s),
            _ => format!("{}:unknown", id.table),
        }
    }

    let task_id_str = record_id_to_string(task.id());
    let list_id_str = record_id_to_string(task_list.id());

    let sql = format!("RELATE {}->belongs_to->{}", task_id_str, list_id_str);
    println!("RELATE SQL: {}", sql);

    // Execute manually and check result
    let result = storage.db.query(&sql).await;
    println!("Query result: {:?}", result);

    // Check what's actually in the belongs_to table
    let belongs_sql = format!("SELECT * FROM belongs_to");
    let mut result2 = storage.db.query(&belongs_sql).await.unwrap();
    let belongs: Vec<serde_json::Value> = result2.take(0).unwrap_or_default();
    println!("All belongs_to records: {:?}", belongs);
}

#[tokio::test]
async fn delete_task_list_deletes_tasks() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create a task list with tasks
    let task_list = storage
        .create_task_list("To Delete".to_string())
        .await
        .expect("Failed to create task list");

    let task1 = storage
        .create_task(
            "Task 1".to_string(),
            "Will be deleted".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            NaiveDate::from_ymd_opt(2026, 5, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .await
        .expect("Failed to create task 1");
    storage
        .relate_task_to_list(task1.id(), task_list.id())
        .await
        .expect("Failed to relate task 1");

    let task2 = storage
        .create_task(
            "Task 2".to_string(),
            "Also will be deleted".to_string(),
            Priority::Low,
            TimeDelta::hours(2),
            NaiveDate::from_ymd_opt(2026, 5, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .await
        .expect("Failed to create task 2");
    storage
        .relate_task_to_list(task2.id(), task_list.id())
        .await
        .expect("Failed to relate task 2");

    // Verify tasks exist before deletion
    let tasks_before = storage
        .get_tasks_in_list(task_list.id())
        .await
        .expect("Failed to get tasks");
    assert_eq!(tasks_before.len(), 2, "Should have 2 tasks before delete");

    // Delete the task list (should also delete tasks)
    storage
        .delete_task_list(task_list.id())
        .await
        .expect("Failed to delete task list");

    // Verify list is gone
    let result = storage.read_task_list(task_list.id()).await;
    assert!(result.is_err(), "List should be deleted");

    // Verify tasks are also gone by checking they don't appear in any list
    let all_lists = storage
        .get_all_task_lists_with_tasks()
        .await
        .expect("Failed to get all lists");

    let found_task = all_lists
        .iter()
        .flat_map(|l| l.tasks.iter())
        .find(|t| t.name() == "Task 1");
    assert!(found_task.is_none(), "Task 1 should be deleted");
}

#[tokio::test]
async fn get_uncompleted_tasks_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let future_date = now + chrono::Duration::days(7);

    // Create an uncompleted task with future deadline
    let _task1 = storage
        .create_task(
            "Task 1".to_string(),
            "Uncompleted".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            future_date,
        )
        .await
        .expect("Failed to create task 1");

    // Create another uncompleted task
    let _task2 = storage
        .create_task(
            "Task 2".to_string(),
            "Also uncompleted".to_string(),
            Priority::High,
            TimeDelta::hours(2),
            future_date,
        )
        .await
        .expect("Failed to create task 2");

    let tasks = storage
        .get_uncompleted_tasks()
        .await
        .expect("Failed to get uncompleted tasks");

    assert_eq!(tasks.len(), 2, "Should return 2 uncompleted tasks");
}

#[tokio::test]
async fn get_uncompleted_tasks_excludes_completed() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let future_date = now + chrono::Duration::days(7);

    // Create an uncompleted task
    let _task1 = storage
        .create_task(
            "Task 1".to_string(),
            "Uncompleted".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            future_date,
        )
        .await
        .expect("Failed to create task 1");

    // Create another task, then mark it as completed
    let task2 = storage
        .create_task(
            "Task 2".to_string(),
            "Will be completed".to_string(),
            Priority::High,
            TimeDelta::hours(2),
            future_date,
        )
        .await
        .expect("Failed to create task 2");

    let mut updated_task2 = task2;
    updated_task2.set_completed(true);
    storage
        .update_task(updated_task2)
        .await
        .expect("Failed to complete task 2");

    let tasks = storage
        .get_uncompleted_tasks()
        .await
        .expect("Failed to get uncompleted tasks");

    assert_eq!(tasks.len(), 1, "Should return only 1 uncompleted task");
    assert_eq!(tasks[0].name(), "Task 1");
}

#[tokio::test]
async fn get_uncompleted_tasks_excludes_overdue() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let future_date = now + chrono::Duration::days(7);
    let past_date = now - chrono::Duration::days(1);

    // Create an uncompleted task with future deadline
    let _task1 = storage
        .create_task(
            "Task 1".to_string(),
            "Future deadline".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            future_date,
        )
        .await
        .expect("Failed to create task 1");

    // Create task with past deadline (overdue)
    let _task2 = storage
        .create_task(
            "Task 2".to_string(),
            "Overdue".to_string(),
            Priority::High,
            TimeDelta::hours(2),
            past_date,
        )
        .await
        .expect("Failed to create task 2");

    let tasks = storage
        .get_uncompleted_tasks()
        .await
        .expect("Failed to get uncompleted tasks");

    assert_eq!(
        tasks.len(),
        1,
        "Should return only 1 task (exclude overdue)"
    );
    assert_eq!(tasks[0].name(), "Task 1");
}

#[tokio::test]
async fn get_future_slots_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let future_date = (now.date() + chrono::Duration::days(1))
        .and_hms_opt(10, 0, 0)
        .unwrap();
    let further_date = (now.date() + chrono::Duration::days(2))
        .and_hms_opt(14, 0, 0)
        .unwrap();

    // Create a future slot
    let _slot1 = storage
        .create_slot(future_date, future_date + chrono::Duration::hours(2))
        .await
        .expect("Failed to create slot 1");

    // Create another future slot
    let _slot2 = storage
        .create_slot(further_date, further_date + chrono::Duration::hours(2))
        .await
        .expect("Failed to create slot 2");

    let slots = storage
        .get_future_slots()
        .await
        .expect("Failed to get future slots");

    assert_eq!(slots.len(), 2, "Should return 2 future slots");
}

#[tokio::test]
async fn get_future_slots_excludes_past() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let past_date = (now.date() - chrono::Duration::days(1))
        .and_hms_opt(10, 0, 0)
        .unwrap();
    let future_date = (now.date() + chrono::Duration::days(1))
        .and_hms_opt(14, 0, 0)
        .unwrap();

    // Create a past slot
    let _slot1 = storage
        .create_slot(past_date, past_date + chrono::Duration::hours(2))
        .await
        .expect("Failed to create slot 1");

    // Create a future slot
    let _slot2 = storage
        .create_slot(future_date, future_date + chrono::Duration::hours(2))
        .await
        .expect("Failed to create slot 2");

    let slots = storage
        .get_future_slots()
        .await
        .expect("Failed to get future slots");

    assert_eq!(slots.len(), 1, "Should return only 1 future slot");
}

#[tokio::test]
async fn delete_task_slot_relations_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    // Create tasks
    let now = chrono::Utc::now().naive_utc();
    let future_date = (now.date() + chrono::Duration::days(1))
        .and_hms_opt(10, 0, 0)
        .unwrap();

    let task1 = storage
        .create_task(
            "Task 1".to_string(),
            "Test task 1".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            future_date + chrono::Duration::days(2),
        )
        .await
        .expect("Failed to create task 1");

    let task2 = storage
        .create_task(
            "Task 2".to_string(),
            "Test task 2".to_string(),
            Priority::High,
            TimeDelta::hours(2),
            future_date + chrono::Duration::days(2),
        )
        .await
        .expect("Failed to create task 2");

    // Delete relations for multiple tasks at once
    storage
        .delete_task_slot_relations(&[task1.id().clone(), task2.id().clone()])
        .await
        .expect("Failed to delete task slot relations");

    // Both tasks should still exist (just no relations)
    let read_task1 = storage
        .read_task(&task1.id())
        .await
        .expect("Failed to read task 1");
    assert_eq!(read_task1.name(), "Task 1");

    let read_task2 = storage
        .read_task(&task2.id())
        .await
        .expect("Failed to read task 2");
    assert_eq!(read_task2.name(), "Task 2");
}

#[tokio::test]
async fn delete_task_slot_relations_no_relations() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let future_date = (now.date() + chrono::Duration::days(1))
        .and_hms_opt(10, 0, 0)
        .unwrap();

    // Create a task without any slot relation
    let task = storage
        .create_task(
            "Task 1".to_string(),
            "Test task".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            future_date + chrono::Duration::days(2),
        )
        .await
        .expect("Failed to create task");

    // Should not panic - just delete nothing
    storage
        .delete_task_slot_relations(&[task.id().clone()])
        .await
        .expect("Failed to delete task slot relations");

    // Task should still exist
    let read_task = storage
        .read_task(&task.id())
        .await
        .expect("Failed to read task");
    assert_eq!(read_task.name(), "Task 1");
}

#[tokio::test]
async fn delete_task_cleans_up_slot_relations() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let future_date = (now.date() + chrono::Duration::days(1))
        .and_hms_opt(10, 0, 0)
        .unwrap();

    // Create a task
    let task = storage
        .create_task(
            "Task to Delete".to_string(),
            "Test task".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            future_date + chrono::Duration::days(2),
        )
        .await
        .expect("Failed to create task");

    // Create a slot
    let slot = storage
        .create_slot(future_date, future_date + chrono::Duration::hours(2))
        .await
        .expect("Failed to create slot");

    // Relate task to slot
    storage
        .relate_task_to_slot(&slot.id(), &task.id(), future_date)
        .await
        .expect("Failed to relate task to slot");

    // Delete the task - this should clean up the relation without errors
    storage
        .delete_task(task.id())
        .await
        .expect("Failed to delete task");

    // Verify task is deleted
    let result = storage.read_task(task.id()).await;
    assert!(result.is_err(), "Task should be deleted");

    // Verify slot still exists
    let slot_check = storage.read_slot(slot.id()).await;
    assert!(slot_check.is_ok(), "Slot should still exist");
}

#[tokio::test]
async fn delete_slot_cleans_up_contains_relations() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let future_date = (now.date() + chrono::Duration::days(1))
        .and_hms_opt(10, 0, 0)
        .unwrap();

    // Create tasks
    let task1 = storage
        .create_task(
            "Task 1".to_string(),
            "Test task 1".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            future_date + chrono::Duration::days(2),
        )
        .await
        .expect("Failed to create task 1");

    let task2 = storage
        .create_task(
            "Task 2".to_string(),
            "Test task 2".to_string(),
            Priority::High,
            TimeDelta::hours(2),
            future_date + chrono::Duration::days(2),
        )
        .await
        .expect("Failed to create task 2");

    // Create a slot
    let slot = storage
        .create_slot(future_date, future_date + chrono::Duration::hours(4))
        .await
        .expect("Failed to create slot");

    // Relate both tasks to slot
    storage
        .relate_task_to_slot(&slot.id(), &task1.id(), future_date)
        .await
        .expect("Failed to relate task 1 to slot");
    storage
        .relate_task_to_slot(
            &slot.id(),
            &task2.id(),
            future_date + chrono::Duration::hours(1),
        )
        .await
        .expect("Failed to relate task 2 to slot");

    // Delete the slot - this should clean up relations without errors
    storage
        .delete_slot(slot.id())
        .await
        .expect("Failed to delete slot");

    // Verify slot is deleted
    let result = storage.read_slot(slot.id()).await;
    assert!(result.is_err(), "Slot should be deleted");

    // Verify tasks still exist
    let read_task1 = storage
        .read_task(task1.id())
        .await
        .expect("Failed to read task 1");
    assert_eq!(read_task1.name(), "Task 1");

    let read_task2 = storage
        .read_task(task2.id())
        .await
        .expect("Failed to read task 2");
    assert_eq!(read_task2.name(), "Task 2");
}

#[tokio::test]
async fn blocked_apps_get_empty() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert!(
        apps.is_empty(),
        "Should return empty list when no apps are blocked"
    );
}

#[tokio::test]
async fn blocked_apps_add_and_get() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let app = storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox",
        )
        .await
        .expect("Failed to add blocked app");

    assert_eq!(app.display_name, "Firefox");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert_eq!(apps.len(), 1, "Should have 1 blocked app");
    assert_eq!(apps[0].display_name, "Firefox");
}

#[tokio::test]
async fn blocked_apps_add_multiple() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox",
        )
        .await
        .expect("Failed to add Firefox");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/code")),
            "VS Code",
        )
        .await
        .expect("Failed to add VS Code");

    storage
        .upsert_blocked_app(
            AppIdentifier::BundleId("com.spotify.client".to_string()),
            "Spotify",
        )
        .await
        .expect("Failed to add Spotify");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert_eq!(apps.len(), 3, "Should have 3 blocked apps");
}

#[tokio::test]
async fn blocked_apps_upsert_updates_existing() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox",
        )
        .await
        .expect("Failed to add Firefox");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox Updated",
        )
        .await
        .expect("Failed to update Firefox");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert_eq!(apps.len(), 1, "Should still have 1 app (not 2)");
    assert_eq!(
        apps[0].display_name, "Firefox Updated",
        "Name should be updated"
    );
}

#[tokio::test]
async fn blocked_apps_delete_all() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox",
        )
        .await
        .expect("Failed to add Firefox");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/code")),
            "VS Code",
        )
        .await
        .expect("Failed to add VS Code");

    storage
        .delete_all_blocked_apps()
        .await
        .expect("Failed to delete all blocked apps");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert!(apps.is_empty(), "Should be empty after delete all");
}

#[tokio::test]
async fn blocked_apps_path_and_bundle_id() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    storage
        .upsert_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox (path)",
        )
        .await
        .expect("Failed to add path-based app");

    storage
        .upsert_blocked_app(
            AppIdentifier::BundleId("com.spotify.client".to_string()),
            "Spotify (bundle)",
        )
        .await
        .expect("Failed to add bundle ID app");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert_eq!(
        apps.len(),
        2,
        "Should have 2 apps with different identifier types"
    );
}

#[tokio::test]
async fn blocked_apps_old_add_method() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let app = storage
        .add_blocked_app(
            AppIdentifier::Path(PathBuf::from("/usr/bin/firefox")),
            "Firefox",
        )
        .await
        .expect("Failed to add blocked app");

    assert_eq!(app.display_name, "Firefox");

    let apps = storage
        .get_blocked_apps()
        .await
        .expect("Failed to get blocked apps");
    assert_eq!(apps.len(), 1, "Should have 1 blocked app");
}

#[tokio::test]
async fn get_next_scheduled_task_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let slot_date = NaiveDate::from_ymd_opt(2028, 5, 1).unwrap();

    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap(),
            slot_date.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot");

    let task1 = storage
        .create_task(
            "Task 1".to_string(),
            "First task".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create task 1");

    let task2 = storage
        .create_task(
            "Task 2".to_string(),
            "Second task".to_string(),
            Priority::High,
            TimeDelta::hours(2),
            slot_date.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create task 2");

    storage
        .relate_task_to_slot(
            &slot.id(),
            &task1.id(),
            slot_date.and_hms_opt(11, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to relate task 1 to slot");

    storage
        .relate_task_to_slot(
            &slot.id(),
            &task2.id(),
            slot_date.and_hms_opt(10, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to relate task 2 to slot");

    let result = storage
        .get_next_scheduled_task()
        .await
        .expect("Failed to get next scheduled task");

    assert!(result.is_some(), "Should return a task");
    let (task, scheduled_for) = result.unwrap();

    assert_eq!(
        task.name(),
        "Task 2",
        "Should return earliest scheduled task"
    );
    assert_eq!(
        scheduled_for,
        slot_date
            .and_hms_opt(10, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp()
    );
}

#[tokio::test]
async fn get_next_scheduled_task_empty() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let result = storage
        .get_next_scheduled_task()
        .await
        .expect("Failed to get next scheduled task");

    assert!(
        result.is_none(),
        "Should return None when no scheduled tasks"
    );
}

#[tokio::test]
async fn get_next_scheduled_task_past_only() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let slot_date = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();

    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap(),
            slot_date.and_hms_opt(12, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create slot");

    let task = storage
        .create_task(
            "Past Task".to_string(),
            "Already passed".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to create task");

    storage
        .relate_task_to_slot(
            &slot.id(),
            &task.id(),
            slot_date.and_hms_opt(10, 0, 0).unwrap(),
        )
        .await
        .expect("Failed to relate task to slot");

    let result = storage
        .get_next_scheduled_task()
        .await
        .expect("Failed to get next scheduled task");

    assert!(
        result.is_none(),
        "Should return None when only past tasks exist"
    );
}
