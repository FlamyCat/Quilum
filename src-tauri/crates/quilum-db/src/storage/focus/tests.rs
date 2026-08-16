use chrono::{NaiveDate, TimeDelta};

use crate::{Storage, task::Priority};

#[tokio::test]
async fn get_active_session_empty() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let session = storage
        .get_active_session()
        .await
        .expect("Failed to get active session");

    assert!(session.is_none(), "Should return None when no tasks exist");
}

#[tokio::test]
async fn get_active_session_ignores_unscheduled() {
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let slot_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    let _task = storage
        .create_task(
            "Unscheduled Task".to_string(),
            "Never scheduled, scheduled_for is NONE".to_string(),
            Priority::Medium,
            TimeDelta::hours(1),
            slot_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await
        .expect("Failed to create task");

    let session = storage
        .get_active_session()
        .await
        .expect("Failed to get active session");

    assert!(
        session.is_none(),
        "Should return None and not error on tasks without scheduled_for"
    );
}
