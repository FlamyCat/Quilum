use crate::{
    db::Storage,
    scheduler::Scheduler,
};
use crate::model::{
    slot::Slot,
    task::{Priority, Task},
};
use chrono::{NaiveDateTime, TimeDelta};
use std::collections::BTreeSet;
use test_helpers::{create_date, create_date_time};
use tokio;

pub(in crate::scheduler) mod test_helpers {
    use chrono::{NaiveDate, NaiveDateTime};

    pub(in crate::scheduler) fn create_date(year: i32, month: u32, day: u32) -> NaiveDateTime {
        NaiveDateTime::from(NaiveDate::from_ymd_opt(year, month, day).unwrap())
    }

    pub(in crate::scheduler) fn create_date_time(
        year: i32,
        month: u32,
        day: u32,
        hours: u32,
        minutes: u32,
    ) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hours, minutes, 0)
            .unwrap()
    }
}

fn create_task(
    index: i32,
    duration: TimeDelta,
    deadline: NaiveDateTime,
    priority: Priority,
) -> Task {
    let name = format!("Задача {index}");
    let description = format!("Описание для задачи {index}");

    Task {
        id: surrealdb::types::RecordId::new("task", format!("{index}").as_str()),
        name,
        description,
        priority,
        estimated_duration: duration.num_seconds(),
        deadline: deadline.and_utc().timestamp(),
    }
}

fn create_slot(start: NaiveDateTime, end: NaiveDateTime) -> Slot {
    Slot {
        id: surrealdb::types::RecordId::new("slot", "test"),
        starts_at: start.and_utc().timestamp(),
        ends_at: end.and_utc().timestamp(),
    }
}

#[tokio::test]
async fn tasks_fit_into_two_slots() {
    let task_1 = create_task(
        1,
        TimeDelta::minutes(40),
        create_date(2025, 6, 2),
        Priority::default(),
    );

    let task_2 = create_task(
        2,
        TimeDelta::minutes(20),
        create_date(2025, 6, 2),
        Priority::default(),
    );

    let tasks = [task_1.clone(), task_2];

    let slots = [
        create_slot(
            create_date_time(2025, 6, 1, 15, 00),
            create_date_time(2025, 6, 1, 15, 30),
        ),
        create_slot(
            create_date_time(2025, 6, 1, 16, 00),
            create_date_time(2025, 6, 1, 16, 30),
        ),
        create_slot(
            create_date_time(2025, 6, 1, 17, 00),
            create_date_time(2025, 6, 1, 17, 40),
        ),
    ];

    let now = create_date_time(2025, 6, 1, 16, 10);
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let scheduler = Scheduler::new(&tasks, &slots, now, &storage);
    let plan = scheduler.schedule();

    assert_eq!(
        plan.tasks().len(),
        2,
        "Обе задачи должны быть включены в план"
    );

    for (task_id, _slot_id, scheduled_for) in plan.tasks() {
        if *task_id == task_1.id().clone() {
            assert_eq!(*scheduled_for, create_date_time(2025, 6, 1, 17, 00));
        } else {
            assert_eq!(*scheduled_for, create_date_time(2025, 6, 1, 16, 10));
        }
    }
}

#[tokio::test]
async fn tasks_fit_into_one_slot() {
    let task_1 = create_task(
        1,
        TimeDelta::minutes(20),
        create_date(2025, 6, 2),
        Priority::default(),
    );

    let task_2 = create_task(
        2,
        TimeDelta::minutes(20),
        create_date(2025, 6, 2),
        Priority::default(),
    );

    let tasks = [task_1, task_2];

    let slots = [
        create_slot(
            create_date_time(2025, 6, 1, 15, 00),
            create_date_time(2025, 6, 1, 15, 40),
        ),
        create_slot(
            create_date_time(2025, 6, 1, 16, 00),
            create_date_time(2025, 6, 1, 17, 00),
        ),
    ];

    let now = create_date_time(2025, 6, 1, 14, 00);
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let scheduler = Scheduler::new(&tasks, &slots, now, &storage);
    let plan = scheduler.schedule();

    let actual: BTreeSet<_> = plan.tasks().iter().map(|(_, _, t)| *t).collect();
    let expected = BTreeSet::from([
        create_date_time(2025, 6, 1, 15, 00),
        create_date_time(2025, 6, 1, 15, 20),
    ]);

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn overdue_tasks_are_not_scheduled() {
    let slots = [
        create_slot(
            create_date_time(2025, 6, 1, 15, 00),
            create_date_time(2025, 6, 1, 15, 20),
        ),
        create_slot(
            create_date_time(2025, 6, 1, 16, 00),
            create_date_time(2025, 6, 1, 16, 40),
        ),
    ];

    let task_1 = create_task(
        1,
        TimeDelta::minutes(20),
        create_date(2025, 6, 2),
        Priority::default(),
    );

    let task_2 = create_task(
        2,
        TimeDelta::minutes(30),
        create_date_time(2025, 6, 1, 15, 45),
        Priority::default(),
    );

    let tasks = [task_1.clone(), task_2.clone()];
    let now = create_date_time(2025, 6, 1, 14, 00);
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let scheduler = Scheduler::new(&tasks, &slots, now, &storage);
    let plan = scheduler.schedule();

    assert_eq!(
        plan.tasks().len(),
        1,
        "В плане должна оказаться только та задача, которую можно успеть сделать в срок"
    );

    let first_planned_task = plan.tasks().first().expect("Should have a task");
    assert_eq!(first_planned_task.0, task_1.id().clone());
    assert_eq!(
        first_planned_task.2,
        create_date_time(2025, 6, 1, 15, 00)
    );

    assert_eq!(
        plan.discarded_tasks().len(),
        1,
        "Одна из задач должна быть отклонена"
    );

    assert_eq!(
        plan.discarded_tasks()[0],
        task_2.id().clone(),
        "Задача с истекшим сроком должна быть отклонена"
    );
}

#[tokio::test]
async fn priority_is_handled_correctly() {
    let slots = [create_slot(
        create_date_time(2025, 6, 1, 15, 00),
        create_date_time(2025, 6, 1, 16, 00),
    )];

    let task_1 = create_task(
        1,
        TimeDelta::hours(1),
        create_date(2025, 6, 2),
        Priority::High,
    );

    let task_2 = create_task(
        2,
        TimeDelta::hours(1),
        create_date(2025, 6, 2),
        Priority::Low,
    );

    let tasks = [task_1.clone(), task_2.clone()];
    let now = create_date_time(2025, 6, 1, 14, 00);
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let scheduler = Scheduler::new(&tasks, &slots, now, &storage);
    let plan = scheduler.schedule();

    assert_eq!(
        plan.tasks().len(),
        1,
        "За отведенное время можно выполнить только одну задачу - с наивысшим приоритетом"
    );

    assert_eq!(
        plan.tasks()[0].0,
        task_1.id().clone(),
        "Задача с более высоким приоритетом должна быть добавлена в план"
    );

    assert_eq!(
        plan.tasks()[0].2,
        create_date_time(2025, 6, 1, 15, 00),
        "Задача должна быть запланирована на начало слота"
    );

    assert_eq!(
        plan.discarded_tasks().len(),
        1,
        "Одна из задач должна быть отклонена"
    );

    assert_eq!(
        plan.discarded_tasks()[0],
        task_2.id().clone(),
        "Задача с более низким приоритетом должна быть отклонена"
    );
}

#[tokio::test]
async fn the_task_that_can_be_done_on_time_should_be_prioritized() {
    let task_1 = create_task(
        1,
        TimeDelta::hours(1),
        create_date_time(2025, 6, 1, 15, 30),
        Priority::High,
    );

    let task_2 = create_task(
        2,
        TimeDelta::hours(1),
        create_date(2025, 6, 2),
        Priority::Low,
    );

    let tasks = [task_1.clone(), task_2.clone()];
    let slots = [create_slot(
        create_date_time(2025, 6, 1, 15, 00),
        create_date_time(2025, 6, 1, 16, 00),
    )];

    let now = create_date_time(2025, 6, 1, 14, 00);
    let storage = Storage::new_mem().await.expect("Failed to create storage");

    let scheduler = Scheduler::new(&tasks, &slots, now, &storage);
    let plan = scheduler.schedule();

    assert_eq!(
        plan.tasks().len(),
        1,
        "Одну из задач можно успеть, она должна быть запланирована"
    );

    assert_eq!(
        plan.tasks()[0].2,
        create_date_time(2025, 6, 1, 15, 00),
        "Задача должна быть запланирована на начало слота"
    );

    assert_eq!(
        plan.tasks()[0].0,
        task_2.id().clone(),
        "Можно успеть только вторую задачу"
    );

    assert_eq!(
        plan.discarded_tasks().len(),
        1,
        "Одну из задач нельзя успеть сделать в срок"
    );

    assert_eq!(
        plan.discarded_tasks()[0],
        task_1.id().clone(),
        "Невозможно успеть сделать первую задачу"
    );
}
