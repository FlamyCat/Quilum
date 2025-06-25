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

    /// Состояние, используемое при рекурсивном составлении плана.
    #[derive(Clone, Debug)]
    pub(super) struct State<'a> {
        table: BTreeMap<TimeDelta, BTreeSet<&'a Task>>,
        plan: Plan<'a>,
        slots: VecDeque<&'a Slot>,
        now: NaiveDateTime,
    }

    impl<'a> State<'a> {
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

        /// Функция ищет первый слот, в котором будет достаточно времени,
        /// чтобы выполнить самую короткую задачу. Слоты до найденного будут удалены.
        ///
        /// Функция обновляет поле ``now``.
        ///
        /// Функция вернет ``None``, если подходящего слота не нашлось.
        /// При этом все слоты будут удалены.
        ///
        pub(super) fn get_available_time(&mut self) -> Option<TimeDelta> {
            let min_duration = *self.table.first_key_value().unwrap().0;

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

            self.slots.front().copied().map(|slot| {
                self.now = cmp::max(self.now, slot.starts_at());

                slot.ends_at() - self.now
            })
        }

        /// Функция удаляет из таблицы записи по ключам, по которым не осталось задач.
        ///
        pub(super) fn remove_empty_rows(&mut self) {
            self.table.retain(|_, task_set| !task_set.is_empty());
        }

        pub(super) fn remove_overdue_tasks(&mut self) {
            self.table.values_mut().for_each(|task_set| {
                task_set.retain(|&task| task.deadline() >= self.now + task.estimated_duration())
            });
        }

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

        fn construct_duration_table(tasks: &'a [Task]) -> BTreeMap<TimeDelta, BTreeSet<&'a Task>> {
            tasks
                .iter()
                .fold(BTreeMap::new(), |mut table, task| {
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
}
