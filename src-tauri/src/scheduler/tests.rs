use super::*;
use crate::model::task::{Priority, ScheduledTask};
use chrono::{NaiveDateTime, TimeDelta};
use std::collections::BTreeSet;
use test_helpers::{create_date, create_date_time};

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
    Task::new(
        format!("Задача {index}"),
        format!("Описание для задачи {index}"),
        priority,
        duration,
        deadline,
    )
}

#[test]
fn tasks_fit_into_two_slots() {
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
        Slot::new(
            create_date_time(2025, 6, 1, 15, 00),
            create_date_time(2025, 6, 1, 15, 30),
        ),
        Slot::new(
            create_date_time(2025, 6, 1, 16, 00),
            create_date_time(2025, 6, 1, 16, 30),
        ),
        Slot::new(
            create_date_time(2025, 6, 1, 17, 00),
            create_date_time(2025, 6, 1, 17, 40),
        ),
    ];

    let now = create_date_time(2025, 6, 1, 16, 10);

    let scheduler = Scheduler::new(&tasks, &slots, now);
    let plan = scheduler.schedule();

    assert_eq!(
        plan.tasks().len(),
        2,
        "Обе задачи должны быть включены в план"
    );

    for task in plan.tasks() {
        if *task.task() == task_1 {
            assert_eq!(task.scheduled_for(), create_date_time(2025, 6, 1, 17, 00));
        } else {
            assert_eq!(task.scheduled_for(), create_date_time(2025, 6, 1, 16, 10));
        }
    }
}

#[test]
fn tasks_fit_into_one_slot() {
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
        Slot::new(
            create_date_time(2025, 6, 1, 15, 00),
            create_date_time(2025, 6, 1, 15, 40),
        ),
        Slot::new(
            create_date_time(2025, 6, 1, 16, 00),
            create_date_time(2025, 6, 1, 17, 00),
        ),
    ];

    let now = create_date_time(2025, 6, 1, 14, 00);

    let scheduler = Scheduler::new(&tasks, &slots, now);
    let plan = scheduler.schedule();

    let actual: BTreeSet<NaiveDateTime> = plan
        .tasks()
        .iter()
        .map(ScheduledTask::scheduled_for)
        .collect();

    let expected = BTreeSet::from([
        create_date_time(2025, 6, 1, 15, 00),
        create_date_time(2025, 6, 1, 15, 20),
    ]);

    assert_eq!(actual, expected);
}

#[test]
fn overdue_tasks_are_not_scheduled() {
    let slots = [
        Slot::new(
            create_date_time(2025, 6, 1, 15, 00),
            create_date_time(2025, 6, 1, 15, 20),
        ),
        Slot::new(
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

    let tasks = [task_1.clone(), task_2];

    let now = create_date_time(2025, 6, 1, 14, 00);

    let scheduler = Scheduler::new(&tasks, &slots, now);
    let plan = scheduler.schedule();

    assert_eq!(
        plan.tasks().len(),
        1,
        "В плане должна оказаться только та задача, которую можно успеть сделать в срок"
    );

    let first_planned_task = plan.tasks().first().unwrap();

    assert_eq!(*first_planned_task.task(), task_1);
    assert_eq!(
        first_planned_task.scheduled_for(),
        create_date_time(2025, 6, 1, 15, 00)
    );
}

#[test]
fn priority_is_handled_correctly() {
    let slots = [Slot::new(
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

    let scheduler = Scheduler::new(&tasks, &slots, now);
    let plan = scheduler.schedule();

    assert_eq!(
        plan.tasks().len(),
        1,
        "За отведенное время можно выполнить только одну задачу - с наивысшим приоритетом"
    );

    assert_eq!(
        *plan[0].task(),
        task_1,
        "Задача с более высоким приоритетом должна быть добавлена в план"
    );

    assert_eq!(
        plan[0].scheduled_for(),
        create_date_time(2025, 6, 1, 15, 00),
        "Задача должна быть запланирована на начало слота"
    );

    assert_eq!(
        plan.discarded_tasks().len(),
        1,
        "Одна из задач должна быть отклонена"
    );

    assert_eq!(
        *plan.discarded_tasks()[0],
        task_2,
        "Задача с более низким приоритетом должна быть отклонена"
    );
}

#[test]
fn the_task_that_can_be_done_on_time_should_be_prioritized() {
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

    let slots = [Slot::new(
        create_date_time(2025, 6, 1, 15, 00),
        create_date_time(2025, 6, 1, 16, 00),
    )];

    let now = create_date_time(2025, 6, 1, 14, 00);

    let scheduler = Scheduler::new(&tasks, &slots, now);
    let plan = scheduler.schedule();

    assert_eq!(
        plan.tasks().len(),
        1,
        "Одну из задач можно успеть, она должна быть запланирована"
    );

    assert_eq!(
        plan[0].scheduled_for(),
        create_date_time(2025, 6, 1, 15, 00),
        "Задача должна быть запланирована на начало слота"
    );
    assert_eq!(*plan[0].task(), task_2, "Можно успеть только вторую задачу");

    assert_eq!(
        plan.discarded_tasks().len(),
        1,
        "Одну из задач нельзя успеть сделать в срок"
    );
    assert_eq!(
        *plan.discarded_tasks()[0],
        task_1,
        "Невозможно успеть сделать первую задачу"
    );
}
