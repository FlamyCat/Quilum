use crate::{
    Storage,
    storage::model::task::Priority
};
use chrono::TimeDelta;

#[tokio::test]
async fn get_uncompleted_tasks_basic() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let now = chrono::Utc::now().naive_utc();
    let future_date = now + chrono::Duration::days(7);

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

    let _slot1 = storage
        .create_slot(future_date, future_date + chrono::Duration::hours(2))
        .await
        .expect("Failed to create slot 1");

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

    let _slot1 = storage
        .create_slot(past_date, past_date + chrono::Duration::hours(2))
        .await
        .expect("Failed to create slot 1");

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
