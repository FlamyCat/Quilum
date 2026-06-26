use crate::{Storage, task::Priority};
use chrono::{NaiveDate, TimeDelta};

#[tokio::test]
async fn task_list_crud() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let task_list = storage
        .create_task_list("My Tasks".to_string())
        .await
        .expect("Failed to create task list");

    assert_eq!(task_list.title, "My Tasks");
    assert!(!format!("{}", task_list.id().table).is_empty());

    let read_list = storage
        .read_task_list(task_list.id())
        .await
        .expect("Failed to read task list");
    assert_eq!(read_list.title, "My Tasks");

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

    let task_list = storage
        .create_task_list("Work Tasks".to_string())
        .await
        .expect("Failed to create task list");

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

    storage
        .relate_task_to_list(task.id(), task_list.id())
        .await
        .expect("Failed to relate task to list");

    let tasks = storage
        .get_tasks_in_list(task_list.id())
        .await
        .expect("Failed to get tasks in list");

    assert_eq!(tasks.len(), 1, "Should have 1 task");
    assert_eq!(tasks[0].name(), "Implement feature");
}

#[tokio::test]
async fn get_all_task_lists_with_tasks() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

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

    let _list3 = storage
        .create_task_list("Empty List".to_string())
        .await
        .expect("Failed to create empty list");

    let all_lists = storage
        .get_all_task_lists_with_tasks()
        .await
        .expect("Failed to get all task lists");

    assert_eq!(all_lists.len(), 3, "Should have 3 lists");

    let found_list1 = all_lists
        .iter()
        .find(|l| l.list.title() == "List 1")
        .expect("List 1 not found");
    assert_eq!(found_list1.tasks.len(), 2, "List 1 should have 2 tasks");

    let found_list2 = all_lists
        .iter()
        .find(|l| l.list.title() == "List 2")
        .expect("List 2 not found");
    assert_eq!(found_list2.tasks.len(), 1, "List 2 should have 1 task");

    let found_list3 = all_lists
        .iter()
        .find(|l| l.list.title() == "Empty List")
        .expect("Empty list not found");
    assert_eq!(found_list3.tasks.len(), 0, "Empty list should have 0 tasks");
}

#[tokio::test]
async fn delete_task_list_deletes_tasks() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

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

    let tasks_before = storage
        .get_tasks_in_list(task_list.id())
        .await
        .expect("Failed to get tasks");
    assert_eq!(tasks_before.len(), 2, "Should have 2 tasks before delete");

    storage
        .delete_task_list(task_list.id())
        .await
        .expect("Failed to delete task list");

    let result = storage.read_task_list(task_list.id()).await;
    assert!(result.is_err(), "List should be deleted");

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
