use chrono::{NaiveDate, TimeDelta};

use crate::{
    Storage,
    task::{Priority, Task},
};

#[tokio::test]
async fn get_events_for_date_range_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    let _event_a = storage
        .create_event(
            "Event A".to_string(),
            "On May 1".to_string(),
            date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
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
                .unwrap()
                .and_utc(),
            NaiveDate::from_ymd_opt(2026, 5, 3)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap()
                .and_utc(),
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
                .unwrap()
                .and_utc(),
            NaiveDate::from_ymd_opt(2026, 5, 5)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap()
                .and_utc(),
        )
        .await
        .expect("Failed to create event C");

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

    let _event = storage
        .create_event(
            "Multi-day Event".to_string(),
            "Spans 3 days".to_string(),
            NaiveDate::from_ymd_opt(2026, 5, 1)
                .unwrap()
                .and_hms_opt(20, 0, 0)
                .unwrap()
                .and_utc(),
            NaiveDate::from_ymd_opt(2026, 5, 3)
                .unwrap()
                .and_hms_opt(6, 0, 0)
                .unwrap()
                .and_utc(),
        )
        .await
        .expect("Failed to create event");

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

    storage
        .create_event(
            "Event 1".to_string(),
            "On May 1".to_string(),
            date1.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            date1.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create event 1");

    storage
        .create_event(
            "Event 2".to_string(),
            "On May 2".to_string(),
            date2.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            date2.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create event 2");

    storage
        .create_event(
            "Event 3".to_string(),
            "On May 3".to_string(),
            date3.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            date3.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create event 3");

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

    for i in 1..=3 {
        storage
            .create_event(
                format!("Event {}", i),
                format!("On May 1, event {}", i),
                date.and_hms_opt(i as u32 * 2, 0, 0).unwrap().and_utc(),
                date.and_hms_opt(i as u32 * 2 + 1, 0, 0).unwrap().and_utc(),
            )
            .await
            .expect("Failed to create event");
    }

    let events = storage
        .get_events_for_date(date)
        .await
        .expect("Failed to query events");

    assert_eq!(events.len(), 3, "Should return all 3 events");
}

#[tokio::test]
async fn get_scheduled_tasks_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            slot_date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot");

    let task = storage
        .create_task(
            "Test Task".to_string(),
            "A scheduled task".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create task");
    assert_eq!(task.completed(), false);

    let scheduled_for = slot_date.and_hms_opt(10, 30, 0).unwrap();
    storage
        .schedule_task(
            slot.id().clone(),
            task.id().clone(),
            scheduled_for.and_utc(),
        )
        .await
        .expect("Failed to relate task to slot");

    let scheduled_tasks = storage
        .get_scheduled_tasks_for_date_range(slot_date, slot_date + TimeDelta::days(1))
        .await
        .expect("Failed to query scheduled tasks");

    assert_eq!(scheduled_tasks.len(), 1, "Should return 1 scheduled task");
    assert_eq!(scheduled_tasks[0].name(), "Test Task");
    assert_eq!(
        scheduled_tasks[0].scheduled_for().unwrap(),
        scheduled_for.and_utc()
    );
}

#[tokio::test]
async fn get_scheduled_tasks_wrong_date() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            slot_date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot");

    let task = storage
        .create_task(
            "Test Task".to_string(),
            "A scheduled task".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create task");
    assert_eq!(task.completed(), false);

    storage
        .schedule_task(
            slot.id().clone(),
            task.id().clone(),
            slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to relate task to slot");

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

    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            slot_date.and_hms_opt(14, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot");

    let mut expected_scheduled_fors = Vec::new();
    for i in 1..=3 {
        let task = storage
            .create_task(
                format!("Task {}", i),
                format!("Scheduled task {}", i),
                Priority::Medium,
                TimeDelta::hours(1),
                slot_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            )
            .await
            .expect("Failed to create task");
        assert_eq!(task.completed(), false);

        let scheduled_for = slot_date.and_hms_opt(10 + (i - 1) as u32, 0, 0).unwrap();
        storage
            .schedule_task(
                slot.id().clone(),
                task.id().clone(),
                scheduled_for.and_utc(),
            )
            .await
            .expect("Failed to relate task to slot");
        expected_scheduled_fors.push(scheduled_for.and_utc());
    }

    let scheduled_tasks = storage
        .get_scheduled_tasks_for_date_range(slot_date, slot_date + TimeDelta::days(1))
        .await
        .expect("Failed to query scheduled tasks");

    assert_eq!(scheduled_tasks.len(), 3, "Should return all 3 tasks");

    let mut task_names: Vec<&str> = scheduled_tasks.iter().map(|st| st.name()).collect();
    task_names.sort();
    assert_eq!(task_names, vec!["Task 1", "Task 2", "Task 3"]);

    let mut scheduled_fors: Vec<_> = scheduled_tasks
        .iter()
        .map(|st| st.scheduled_for().unwrap())
        .collect();
    scheduled_fors.sort();
    assert_eq!(scheduled_fors, expected_scheduled_fors);
}

#[tokio::test]
async fn get_scheduled_tasks_date_range_filter() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let date1 = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot_a = storage
        .create_slot(
            date1.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            date1.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot A");

    let task_t1 = storage
        .create_task(
            "Task T1".to_string(),
            "In slot A".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            date1.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create T1");
    assert_eq!(task_t1.completed(), false);
    storage
        .schedule_task(
            slot_a.id().clone(),
            task_t1.id().clone(),
            date1.and_hms_opt(10, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to relate T1 to slot A");

    let date2 = NaiveDate::from_ymd_opt(2026, 5, 3).unwrap();
    let slot_b = storage
        .create_slot(
            date2.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            date2.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot B");

    let task_t2 = storage
        .create_task(
            "Task T2".to_string(),
            "In slot B".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            date2.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create T2");
    assert_eq!(task_t2.completed(), false);
    storage
        .schedule_task(
            slot_b.id().clone(),
            task_t2.id().clone(),
            date2.and_hms_opt(10, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to relate T2 to slot B");

    let scheduled_tasks = storage
        .get_scheduled_tasks_for_date_range(date1, date2)
        .await
        .expect("Failed to query scheduled tasks");

    assert_eq!(
        scheduled_tasks.len(),
        1,
        "Should return only T1 (May 3 is excluded)"
    );
    assert_eq!(scheduled_tasks[0].name(), "Task T1");
}

#[tokio::test]
async fn get_scheduled_tasks_empty_result() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

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

    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            slot_date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot");

    let task = storage
        .create_task(
            "Test Task".to_string(),
            "A scheduled task".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create task");
    assert_eq!(task.completed(), false);

    let scheduled_for = slot_date.and_hms_opt(10, 30, 0).unwrap();
    storage
        .schedule_task(
            slot.id().clone(),
            task.id().clone(),
            scheduled_for.and_utc(),
        )
        .await
        .expect("Failed to relate task to slot");

    let slots_with_tasks = storage
        .get_slots_with_tasks_for_date_range(slot_date, slot_date + TimeDelta::days(1))
        .await
        .expect("Failed to query slots with tasks");

    assert_eq!(slots_with_tasks.len(), 1, "Should return 1 slot");
    assert_eq!(
        slots_with_tasks[0].slot.starts_at(),
        slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc()
    );
    assert_eq!(slots_with_tasks[0].tasks.len(), 1, "Should have 1 task");
    assert_eq!(slots_with_tasks[0].tasks[0].name(), "Test Task");
    assert_eq!(
        slots_with_tasks[0].tasks[0]
            .scheduled_for()
            .expect("scheduled_for is missing"),
        scheduled_for.and_utc()
    );
}

#[tokio::test]
async fn get_slots_with_tasks_multiple_tasks() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            slot_date.and_hms_opt(14, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot");

    let mut expected_scheduled_fors = Vec::new();
    for i in 1..=3 {
        let task = storage
            .create_task(
                format!("Task {}", i),
                format!("Scheduled task {}", i),
                Priority::Medium,
                TimeDelta::hours(1),
                slot_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            )
            .await
            .expect("Failed to create task");
        assert_eq!(task.completed(), false);

        let scheduled_for = slot_date.and_hms_opt(10 + (i - 1) as u32, 0, 0).unwrap();
        storage
            .schedule_task(
                slot.id().clone(),
                task.id().clone(),
                scheduled_for.and_utc(),
            )
            .await
            .expect("Failed to relate task to slot");
        expected_scheduled_fors.push(scheduled_for.and_utc());
    }

    let slots_with_tasks = storage
        .get_slots_with_tasks_for_date_range(slot_date, slot_date + TimeDelta::days(1))
        .await
        .expect("Failed to query slots with tasks");

    assert_eq!(slots_with_tasks.len(), 1, "Should return 1 slot");
    assert_eq!(slots_with_tasks[0].tasks.len(), 3, "Should have 3 tasks");

    let mut task_names: Vec<_> = slots_with_tasks[0].tasks.iter().map(Task::name).collect();
    task_names.sort();
    assert_eq!(task_names, vec!["Task 1", "Task 2", "Task 3"]);

    let mut scheduled_fors: Vec<_> = slots_with_tasks[0]
        .tasks
        .iter()
        .map(|t| t.scheduled_for().expect("Scheduled for is missing"))
        .collect();
    scheduled_fors.sort();
    assert_eq!(scheduled_fors, expected_scheduled_fors);
}

#[tokio::test]
async fn get_slots_with_tasks_multiple_slots() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    let slot_a = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            slot_date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
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
                slot_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            )
            .await
            .expect("Failed to create task");
        assert_eq!(task.completed(), false);

        storage
            .schedule_task(
                slot_a.id().clone(),
                task.id().clone(),
                slot_date
                    .and_hms_opt(10 + (i - 1) as u32, 0, 0)
                    .unwrap()
                    .and_utc(),
            )
            .await
            .expect("Failed to relate task to slot A");
    }

    let slot_b = storage
        .create_slot(
            slot_date.and_hms_opt(14, 0, 0).unwrap().and_utc(),
            slot_date.and_hms_opt(16, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot B");

    let task_b = storage
        .create_task(
            "Task B1".to_string(),
            "In slot B".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create task");
    assert_eq!(task_b.completed(), false);

    storage
        .schedule_task(
            slot_b.id().clone(),
            task_b.id().clone(),
            slot_date.and_hms_opt(14, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to relate task to slot B");

    let slots_with_tasks = storage
        .get_slots_with_tasks_for_date_range(slot_date, slot_date + TimeDelta::days(1))
        .await
        .expect("Failed to query slots with tasks");

    assert_eq!(slots_with_tasks.len(), 2, "Should return 2 slots");

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

    let date1 = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let slot_a = storage
        .create_slot(
            date1.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            date1.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot A");

    let task_t1 = storage
        .create_task(
            "Task T1".to_string(),
            "In slot A".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            date1.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create T1");
    storage
        .schedule_task(
            slot_a.id().clone(),
            task_t1.id().clone(),
            date1.and_hms_opt(10, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to relate T1 to slot A");

    let date2 = NaiveDate::from_ymd_opt(2026, 5, 3).unwrap();
    let slot_b = storage
        .create_slot(
            date2.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            date2.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot B");

    let task_t2 = storage
        .create_task(
            "Task T2".to_string(),
            "In slot B".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            date2.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create T2");
    storage
        .schedule_task(
            slot_b.id().clone(),
            task_t2.id().clone(),
            date2.and_hms_opt(10, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to relate T2 to slot B");

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
    assert_eq!(slots_with_tasks[0].tasks[0].name(), "Task T1");
}

#[tokio::test]
async fn get_slots_with_tasks_empty_result() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

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

    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let _slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            slot_date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot");

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

    let _event = storage
        .create_event(
            "Today's Event".to_string(),
            "An event today".to_string(),
            today.and_hms_opt(9, 0, 0).unwrap().and_utc(),
            today.and_hms_opt(10, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create event");

    let slot = storage
        .create_slot(
            today.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            today.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot");

    let task = storage
        .create_task(
            "Today's Task".to_string(),
            "A task for today".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            today.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create task");
    assert_eq!(task.completed(), false);

    let scheduled_for = today.and_hms_opt(10, 30, 0).unwrap();
    storage
        .schedule_task(
            slot.id().clone(),
            task.id().clone(),
            scheduled_for.and_utc(),
        )
        .await
        .expect("Failed to relate task to slot");

    let (events, scheduled_tasks) = storage
        .get_today_timetable(today)
        .await
        .expect("Failed to get today timetable");

    assert_eq!(events.len(), 1, "Should have 1 event");
    assert_eq!(events[0].name(), "Today's Event");
    assert_eq!(scheduled_tasks.len(), 1, "Should have 1 scheduled task");
    assert_eq!(scheduled_tasks[0].name(), "Today's Task");
    assert_eq!(
        scheduled_tasks[0].scheduled_for().unwrap(),
        scheduled_for.and_utc()
    );
}

#[tokio::test]
async fn get_today_timetable_empty() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");
    let today = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();

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

    let _event_a = storage
        .create_event(
            "Event A".to_string(),
            "On May 1".to_string(),
            week_start.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            week_start.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create event A");

    let slot_b = storage
        .create_slot(
            week_start.and_hms_opt(14, 0, 0).unwrap().and_utc(),
            week_start.and_hms_opt(18, 0, 0).unwrap().and_utc(),
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
                week_start.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            )
            .await
            .expect("Failed to create task");
        assert_eq!(task.completed(), false);

        storage
            .schedule_task(
                slot_b.id().clone(),
                task.id().clone(),
                week_start
                    .and_hms_opt(14 + (i - 1) as u32, 0, 0)
                    .unwrap()
                    .and_utc(),
            )
            .await
            .expect("Failed to relate task to slot B");
    }

    let _event_c = storage
        .create_event(
            "Event C".to_string(),
            "On May 5".to_string(),
            NaiveDate::from_ymd_opt(2026, 5, 5)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap()
                .and_utc(),
            NaiveDate::from_ymd_opt(2026, 5, 5)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap()
                .and_utc(),
        )
        .await
        .expect("Failed to create event C");

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

    let _slot_a = storage
        .create_slot(
            week_start.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            week_start.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot A");

    let slot_b_date = NaiveDate::from_ymd_opt(2026, 5, 8).unwrap();
    let slot_b = storage
        .create_slot(
            slot_b_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            slot_b_date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot B");

    let task_b = storage
        .create_task(
            "Task B".to_string(),
            "In slot B (next week)".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_b_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create task");
    assert_eq!(task_b.completed(), false);

    storage
        .schedule_task(
            slot_b.id().clone(),
            task_b.id().clone(),
            slot_b_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to relate task to slot B");

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
async fn get_next_scheduled_task_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let slot_date = NaiveDate::from_ymd_opt(2028, 5, 1).unwrap();

    let slot = storage
        .create_slot(
            slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            slot_date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot");

    let task1 = storage
        .create_task(
            "Task 1".to_string(),
            "First task".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create task 1");

    let task2 = storage
        .create_task(
            "Task 2".to_string(),
            "Second task".to_string(),
            Priority::High,
            TimeDelta::hours(2),
            slot_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create task 2");

    storage
        .schedule_task(
            slot.id().clone(),
            task1.id().clone(),
            slot_date.and_hms_opt(11, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to relate task 1 to slot");

    storage
        .schedule_task(
            slot.id().clone(),
            task2.id().clone(),
            slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to relate task 2 to slot");

    let result = storage
        .get_next_scheduled_task()
        .await
        .expect("Failed to get next scheduled task");

    assert!(result.is_some(), "Should return a task");
    let task = result.unwrap();

    assert_eq!(
        task.name(),
        "Task 2",
        "Should return earliest scheduled task"
    );
    assert_eq!(
        task.scheduled_for().unwrap(),
        slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc()
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
            slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            slot_date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create slot");

    let task = storage
        .create_task(
            "Past Task".to_string(),
            "Already passed".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create task");

    storage
        .schedule_task(
            slot.id().clone(),
            task.id().clone(),
            slot_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
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
