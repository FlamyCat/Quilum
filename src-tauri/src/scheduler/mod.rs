use crate::model::{plan::Plan, slot::Slot, task::Task};
use chrono::Local;
use state::State;

pub(crate) struct Scheduler<'a> {
    tasks: &'a [Task],
    upcoming_slots: &'a [Slot],
}

impl<'a> Scheduler<'a> {
    pub fn new(tasks: &'a [Task], upcoming_slots: &'a [Slot]) -> Self {
        Self {
            tasks,
            upcoming_slots,
        }
    }

    /// Метод составляет план на основе полученных задач
    ///
    pub(crate) fn schedule(&self) -> Plan {
        let state = State::new(
            self.tasks,
            self.upcoming_slots.iter().collect(),
            Local::now().naive_local(),
        );

        Self::schedule_from(state)
    }

    /// Функция, составляющая план из состояния.
    ///
    /// Функция завершает работу в следующих случаях:
    /// * когда не осталось задач, которые можно добавить в план,
    /// * когда не осталось слотов, в которые можно запланировать задачи.
    ///
    /// При возникновении любого из них функция возвращает получившийся план.
    ///
    /// При остальных случаях функция запускается от состояния, в котором запланирована еще
    /// одна задача, помещенная в слот.
    ///
    fn schedule_from(mut state: State) -> Plan {
        state.remove_empty_rows();
        state.remove_overdue_tasks();

        if state.table().is_empty() || state.slots().is_empty() {
            return state.plan();
        }

        let Some(remaining_time) = state.get_available_time() else {
            return state.plan();
        };

        state
            .table()
            .iter()
            .filter(|(&dur, _)| dur <= remaining_time)
            .map(|(_, task_set)| {
                let task = task_set.first().copied().unwrap();
                state.clone().create_next_state(task)
            })
            .map(Self::schedule_from)
            .max_by_key(Plan::score)
            .unwrap()
    }
}

mod state {
    use crate::model::{
        plan::Plan,
        slot::Slot,
        task::{ScheduledTask, Task},
    };
    use chrono::{NaiveDateTime, TimeDelta};
    use std::cmp;
    use std::collections::{BTreeMap, BTreeSet, VecDeque};

    /// Структура, описывающая состояние
    /// (еще не запланированные задачи, текущий план, оставшиеся слоты)
    /// на каждой итерации алгоритма планирования.
    ///
    #[derive(Clone, Debug)]
    pub(super) struct State<'a> {
        table: BTreeMap<TimeDelta, BTreeSet<&'a Task>>,
        plan: Plan<'a>,
        slots: VecDeque<&'a Slot>,
        now: NaiveDateTime,
    }

    impl<'a> State<'a> {
        /// Создает начальный вариант состояния на основе списка задач, слотов и
        /// текущего момента времени.
        ///
        pub(super) fn new(
            tasks: &'a [Task],
            slots: VecDeque<&'a Slot>,
            now: NaiveDateTime,
        ) -> Self {
            let table = Self::construct_duration_table(tasks);

            Self {
                table,
                plan: Plan::new(),
                slots,
                now,
            }
        }

        /// Функция создает следующую фазу состояния, где задача ``task`` добавлена в план.
        ///
        /// Функция не проверяет, возможно ли добавить задачу в план.
        /// Перед ее вызовом необходимо убедиться, что в слоте достаточно времени, чтобы задача могла
        /// быть запланирована.
        ///
        pub(super) fn create_next_state(self, task: &'a Task) -> Self {
            let scheduled_task = ScheduledTask::new(task, self.now);

            Self {
                plan: self.plan.with_task(scheduled_task),
                now: self.now + task.estimated_duration(),
                ..self
            }
        }

        /// Метод ищет первый слот, в котором будет достаточно времени,
        /// чтобы выполнить самую короткую задачу. Слоты до найденного будут удалены из списка.
        ///
        /// Метод обновляет поле ``now`` - она задает его равным началу найденного слота.
        /// Если слота не нашлось, ``now`` не обновляется.
        ///
        /// ## Возвращаемое значение
        /// * Метод вернет ``None``, если подходящего слота не нашлось или список задач пуст.
        ///   При этом список слотов в состоянии будет очищен.
        ///
        /// * Метод вернет ``Some(time_delta)``, если подходящий слот найдется.
        ///   Значение ``time_delta`` будет равняться оставшемуся в слоте времени
        ///   (с учетом текущего момента времени).
        ///
        #[must_use]
        pub(super) fn get_available_time(&mut self) -> Option<TimeDelta> {
            let min_duration = *self.table.first_key_value()?.0;

            self.skip_unsuitable_slots(min_duration);

            self.slots.front().copied().map(|slot| {
                self.now = cmp::max(self.now, slot.starts_at());

                slot.ends_at() - self.now
            })
        }

        /// Метод удаляет ведущие слоты в списке,
        /// * в которых осталось времени меньше, чем ``min_duration``;
        /// * просрочены.
        ///
        fn skip_unsuitable_slots(&mut self, min_duration: TimeDelta) {
            let count = self
                .slots
                .iter()
                .copied()
                .position(|slot| {
                    let latest = cmp::max(self.now, slot.starts_at());
                    slot.ends_at() - latest >= min_duration
                })
                .unwrap_or(self.slots.len());

            self.slots.drain(..count);
        }

        /// Функция удаляет из таблицы записи по ключам, по которым не осталось задач.
        ///
        pub(super) fn remove_empty_rows(&mut self) {
            self.table.retain(|_, task_set| !task_set.is_empty());
        }

        /// Метод удаляет все задачи, которые нельзя успеть выполнить в срок.
        ///
        pub(super) fn remove_overdue_tasks(&mut self) {
            self.table.values_mut().for_each(|task_set| {
                task_set.retain(|&task| task.deadline() >= self.now + task.estimated_duration())
            });

            self.remove_empty_rows();
        }

        /// Метод строит таблицу, которая группирует задачи по отведенному на них времени.
        ///
        fn construct_duration_table(tasks: &'a [Task]) -> BTreeMap<TimeDelta, BTreeSet<&'a Task>> {
            tasks.iter().fold(BTreeMap::new(), |mut table, task| {
                table
                    .entry(task.estimated_duration())
                    .or_default()
                    .insert(task);
                table
            })
        }

        pub(super) fn table(&self) -> &BTreeMap<TimeDelta, BTreeSet<&'a Task>> {
            &self.table
        }

        pub(super) fn plan(self) -> Plan<'a> {
            self.plan
        }

        pub(super) fn slots(&self) -> &VecDeque<&'a Slot> {
            &self.slots
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::model::task::{Priority, Task};
        use chrono::{NaiveDate, NaiveDateTime, TimeDelta};

        fn create_task(index: u32, estimated_duration: TimeDelta) -> Task {
            let name = format!("Задача {index}");
            let description = format!("Описание для задачи {index}");
            let priority = Priority::default();
            let deadline = create_date_time(2025, 6, 1, 16, 20);

            Task::new(name, description, priority, estimated_duration, deadline)
        }

        fn create_date(year: i32, month: u32, day: u32) -> NaiveDateTime {
            NaiveDateTime::from(NaiveDate::from_ymd_opt(year, month, day).unwrap())
        }

        fn create_date_time(
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

        mod skip_unsuitable_slots_tests {
            use super::*;
            use crate::scheduler::*;
            use chrono::TimeDelta;
            use std::collections::VecDeque;

            #[test]
            fn does_nothing_on_empty_list() {
                let tasks = [];
                let slot_queue = VecDeque::new();
                let now = create_date(2025, 6, 1);

                let mut state = State::new(&tasks, slot_queue, now);

                state.skip_unsuitable_slots(TimeDelta::minutes(20));

                assert!(
                    state.slots.is_empty(),
                    "Очередь слотов должен остаться пустым"
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

                state.skip_unsuitable_slots(TimeDelta::minutes(20));

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

                state.skip_unsuitable_slots(TimeDelta::minutes(20));

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

                state.skip_unsuitable_slots(TimeDelta::minutes(20));

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

                let tasks = [];
                let slot = Slot::new(slot_start, slot_end);

                let slot_queue = VecDeque::from([&slot]);

                let mut state = State::new(&tasks, slot_queue, now);

                state.skip_unsuitable_slots(TimeDelta::minutes(20));

                assert_eq!(
                    state.slots.len(),
                    1,
                    "Подходящий слот должен был быть сохранен в очереди"
                );
            }
        }

        mod get_available_time_tests {
            use crate::model::slot::Slot;
            use crate::scheduler::state::tests::create_date_time;
            use crate::scheduler::state::{
                tests::{create_date, create_task},
                State,
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
    }
}
