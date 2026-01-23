use crate::model::task::{Priority, Task};
use crate::scheduler::tests::test_helpers::create_date_time;
use chrono::TimeDelta;

fn create_task(index: u32, estimated_duration: TimeDelta) -> Task {
    let name = format!("Задача {index}");
    let description = format!("Описание для задачи {index}");
    let priority = Priority::default();
    let deadline = create_date_time(2025, 6, 2, 23, 59);

    Task::new(name, description, priority, estimated_duration, deadline)
}

mod skip_unsuitable_slots_tests {
    use crate::scheduler::state::tests::create_task;
    use crate::scheduler::tests::test_helpers::{create_date, create_date_time};
    use crate::scheduler::*;
    use chrono::TimeDelta;
    use std::collections::VecDeque;

    #[test]
    fn does_nothing_on_empty_list() {
        let tasks = [];
        let slot_queue = VecDeque::new();
        let now = create_date(2025, 6, 1);

        let mut state = State::new(&tasks, slot_queue, now);

        state.skip_unsuitable_slots();

        assert!(
            state.slots.is_empty(),
            "Очередь слотов должна остаться пустой"
        );
    }

    #[test]
    fn skips_past_slots() {
        let tasks = [];
        let now = create_date(2025, 6, 1);
        let slot_date_time = create_date(2025, 5, 1);

        let past_slot = Slot::new(slot_date_time, slot_date_time + TimeDelta::minutes(20));

        let slot_queue = VecDeque::from([&past_slot]);

        let mut state = State::new(&tasks, slot_queue, now);

        state.skip_unsuitable_slots();

        assert!(
            state.slots.is_empty(),
            "Единственный просроченный слот должен был быть \
            удален из очереди"
        );
    }

    #[test]
    fn skips_slots_without_enough_time_for_the_task() {
        let now = create_date_time(2025, 6, 1, 16, 10);

        let slot_start = create_date_time(2025, 6, 1, 16, 00);
        let slot_end = create_date_time(2025, 6, 1, 16, 20);

        let tasks = [];
        let slot = Slot::new(slot_start, slot_end);

        let slot_queue = VecDeque::from([&slot]);

        let mut state = State::new(&tasks, slot_queue, now);

        state.skip_unsuitable_slots();

        assert!(
            state.slots.is_empty(),
            "Единственный слот, в котором не осталось времени \
            на задачу, должен был быть удален"
        );
    }

    #[test]
    fn skips_short_slots() {
        let now = create_date_time(2025, 6, 1, 16, 00);

        let slot_start = create_date_time(2025, 6, 1, 16, 10);
        let slot_end = create_date_time(2025, 6, 1, 16, 20);

        let tasks = [];
        let slot = Slot::new(slot_start, slot_end);

        let slot_queue = VecDeque::from([&slot, &slot]);

        let mut state = State::new(&tasks, slot_queue, now);

        state.skip_unsuitable_slots();

        assert!(
            state.slots.is_empty(),
            "Два слишком коротких слота должны были быть удалены"
        );
    }

    #[test]
    fn stops_at_suitable_slot() {
        let now = create_date_time(2025, 6, 1, 16, 00);

        let slot_start = create_date_time(2025, 6, 1, 16, 10);
        let slot_end = create_date_time(2025, 6, 1, 16, 30);

        let tasks = [create_task(1, TimeDelta::minutes(20))];
        let slot = Slot::new(slot_start, slot_end);

        let slot_queue = VecDeque::from([&slot]);

        let mut state = State::new(&tasks, slot_queue, now);

        state.skip_unsuitable_slots();

        assert_eq!(
            state.slots.len(),
            1,
            "Подходящий слот должен был быть сохранен в очереди"
        );
    }
}

mod get_available_time_tests {
    use crate::{
        model::slot::Slot,
        scheduler::{
            state::{tests::create_task, State},
            tests::test_helpers::{create_date, create_date_time},
        },
    };
    use chrono::TimeDelta;
    use std::collections::VecDeque;

    #[test]
    fn no_slots() {
        let slots = VecDeque::new();
        let tasks = [create_task(1, TimeDelta::minutes(20))];
        let now = create_date(2025, 6, 1);

        let mut state = State::new(&tasks, slots, now);

        assert!(state.get_available_time().is_none());
        assert_eq!(state.now, now, "now должно остаться прежним");
    }

    #[test]
    fn short_slot() {
        let slot_start = create_date_time(2025, 6, 1, 16, 30);

        let slot_end = slot_start + TimeDelta::minutes(10);
        let slot = Slot::new(slot_start, slot_end);

        let slots = VecDeque::from([&slot]);

        let tasks = [create_task(1, TimeDelta::minutes(20))];

        let now = create_date_time(2025, 6, 1, 16, 00);

        let mut state = State::new(&tasks, slots, now);

        let available_time = state.get_available_time();
        assert!(available_time.is_none());
        assert_eq!(now, state.now);
    }

    #[test]
    fn suitable_slot_now() {
        let now = create_date_time(2025, 6, 1, 16, 10);

        let slot_start = create_date_time(2025, 6, 1, 16, 00);
        let slot_end = slot_start + TimeDelta::minutes(30);

        let slot = Slot::new(slot_start, slot_end);

        let slot_queue = VecDeque::from([&slot]);

        let tasks = [create_task(1, TimeDelta::minutes(20))];

        let mut state = State::new(&tasks, slot_queue, now);

        let actual_available_time = state
            .get_available_time()
            .expect("В слоте достаточно времени для выполнения задачи");

        let task = state
            .table
            .first_entry()
            .expect("В таблице должна остаться строка с задачей")
            .get()
            .first()
            .copied()
            .expect("В строке должна была остаться задача");

        assert_eq!(state.slots.len(), 1);

        let first_slot = state.slots.front().copied().expect("Слот должен остаться");

        let expected_available_time = first_slot.ends_at() - now;
        assert_eq!(actual_available_time, expected_available_time);
        assert!(actual_available_time >= task.estimated_duration());
        assert_eq!(now, state.now);
    }

    #[test]
    fn upcoming_suitable_slot() {
        let now = create_date_time(2025, 6, 1, 16, 00);

        let slot_start = create_date_time(2025, 6, 1, 16, 10);
        let slot_duration = TimeDelta::minutes(30);
        let slot_end = slot_start + slot_duration;

        let slot = Slot::new(slot_start, slot_end);

        let slot_queue = VecDeque::from([&slot]);

        let tasks = [create_task(1, TimeDelta::minutes(20))];

        let mut state = State::new(&tasks, slot_queue, now);

        let actual_available_time = state
            .get_available_time()
            .expect("В слоте достаточно времени для выполнения задачи");

        let task = state
            .table
            .first_entry()
            .expect("В таблице должна остаться строка с задачей")
            .get()
            .first()
            .copied()
            .expect("В строке должна остаться задача");

        assert_eq!(state.slots.len(), 1);

        let first_slot = state.slots.front().copied().expect("Слот должен остаться");

        let expected_available_time = first_slot.duration();
        assert_eq!(actual_available_time, expected_available_time);
        assert!(actual_available_time >= task.estimated_duration());
        assert_eq!(first_slot.starts_at(), state.now);
    }
}
