use crate::{Storage, task::Priority};
use chrono::TimeDelta;

#[tokio::test]
async fn delete_task_slot_relations_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let future_date = (now.date() + TimeDelta::days(1))
        .and_hms_opt(10, 0, 0)
        .unwrap();

    let task1 = storage
        .create_task(
            "Task 1".to_string(),
            "Test task 1".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            (future_date + TimeDelta::days(2)).and_utc(),
        )
        .await
        .expect("Failed to create task 1");

    let task2 = storage
        .create_task(
            "Task 2".to_string(),
            "Test task 2".to_string(),
            Priority::High,
            TimeDelta::hours(2),
            (future_date + TimeDelta::days(2)).and_utc(),
        )
        .await
        .expect("Failed to create task 2");

    storage
        .unschedule_tasks(
            [task1.id().clone(), task2.id().clone()].into_iter().collect(),
        )
        .await
        .expect("Failed to delete task slot relations");

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
async fn delete_task_slot_relations_no_relations() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let future_date = (now.date() + TimeDelta::days(1))
        .and_hms_opt(10, 0, 0)
        .unwrap();

    let task = storage
        .create_task(
            "Task 1".to_string(),
            "Test task".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            (future_date + TimeDelta::days(2)).and_utc(),
        )
        .await
        .expect("Failed to create task");

    storage
        .unschedule_tasks([task.id().clone()].into_iter().collect())
        .await
        .expect("Failed to delete task slot relations");

    let read_task = storage
        .read_task(task.id())
        .await
        .expect("Failed to read task");
    assert_eq!(read_task.name(), "Task 1");
}

#[tokio::test]
async fn delete_task_cleans_up_slot_relations() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let future_date = (now.date() + TimeDelta::days(1))
        .and_hms_opt(10, 0, 0)
        .unwrap()
        .and_utc();

    let task = storage
        .create_task(
            "Task to Delete".to_string(),
            "Test task".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            future_date + TimeDelta::days(2),
        )
        .await
        .expect("Failed to create task");

    let slot = storage
        .create_slot(future_date, future_date + TimeDelta::hours(2))
        .await
        .expect("Failed to create slot");

    storage
        .schedule_task(slot.id().clone(), task.id().clone(), future_date)
        .await
        .expect("Failed to relate task to slot");

    storage
        .delete_task(task.id())
        .await
        .expect("Failed to delete task");

    let result = storage.read_task(task.id()).await;
    assert!(result.is_err(), "Task should be deleted");

    let slot_check = storage.read_slot(slot.id()).await;
    assert!(slot_check.is_ok(), "Slot should still exist");
}

#[tokio::test]
async fn delete_slot_cleans_up_contains_relations() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let future_date = (now.date() + TimeDelta::days(1))
        .and_hms_opt(10, 0, 0)
        .unwrap()
        .and_utc();

    let task1 = storage
        .create_task(
            "Task 1".to_string(),
            "Test task 1".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            future_date + TimeDelta::days(2),
        )
        .await
        .expect("Failed to create task 1");

    let task2 = storage
        .create_task(
            "Task 2".to_string(),
            "Test task 2".to_string(),
            Priority::High,
            TimeDelta::hours(2),
            future_date + TimeDelta::days(2),
        )
        .await
        .expect("Failed to create task 2");

    let slot = storage
        .create_slot(future_date, future_date + TimeDelta::hours(4))
        .await
        .expect("Failed to create slot");

    storage
        .schedule_task(slot.id().clone(), task1.id().clone(), future_date)
        .await
        .expect("Failed to relate task 1 to slot");
    storage
        .schedule_task(
            slot.id().clone(),
            task2.id().clone(),
            future_date + TimeDelta::hours(1),
        )
        .await
        .expect("Failed to relate task 2 to slot");

    storage
        .delete_slot(slot.id())
        .await
        .expect("Failed to delete slot");

    let result = storage.read_slot(slot.id()).await;
    assert!(result.is_err(), "Slot should be deleted");

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
