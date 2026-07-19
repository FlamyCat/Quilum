use std::{
    cmp,
    collections::{BTreeMap, BTreeSet, VecDeque},
};

use chrono::{NaiveDateTime, TimeDelta};
use quilum_db::storage::model::{
    plan::Plan,
    slot::Slot,
    task::{Task, Unscheduled},
};
use surrealdb::types::RecordId;

/// Обертка вокруг `&Task`, которая реализует `Ord` для использования в `BTreeSet`.
#[derive(Clone, Copy, Debug)]
pub(super) struct TaskRef<'a, S>
where
    S: Ord,
{
    task: &'a Task<S>,
}

impl<'a, S: Ord> TaskRef<'a, S> {
    pub fn new(task: &'a Task<S>) -> Self {
        Self { task }
    }

    pub fn record_id(&self) -> &RecordId {
        &self.task.id()
    }

    pub fn task(&self) -> &Task<S> {
        self.task
    }
}

impl<'a, S: PartialEq + Ord> PartialEq for TaskRef<'a, S> {
    fn eq(&self, other: &Self) -> bool {
        self.task == other.task
    }
}

impl<'a, S: PartialEq + Ord> Eq for TaskRef<'a, S> {}

impl<'a, S: PartialEq + Ord> PartialOrd for TaskRef<'a, S> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, S: PartialEq + Ord> Ord for TaskRef<'a, S> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.task.cmp(other.task)
    }
}

/// Структура, описывающая состояние
/// (еще не запланированные задачи, текущий план, оставшиеся слоты)
/// на каждой итерации алгоритма планирования.
///
#[derive(Clone, Debug)]
pub(super) struct State<'a> {
    table: BTreeMap<TimeDelta, BTreeSet<TaskRef<'a, Unscheduled>>>,
    plan: Plan,
    slots: VecDeque<&'a Slot>,
    now: NaiveDateTime,
    current_slot: Option<&'a Slot>,
}

impl<'a> State<'a> {
    /// Создает начальный вариант состояния на основе списка задач, слотов и
    /// текущего момента времени.
    ///
    pub(super) fn new(
        tasks: &'a [Task<Unscheduled>],
        slots: VecDeque<&'a Slot>,
        now: NaiveDateTime,
    ) -> Self {
        let table = Self::construct_duration_table(tasks);

        Self {
            table,
            plan: Plan::new(),
            slots,
            now,
            current_slot: None,
        }
    }

    pub(super) fn discard_remaining_tasks(&mut self) {
        let remaining_tasks = self
            .table
            .values()
            .flatten()
            .map(|tr| tr.record_id().clone());

        self.plan.discard_tasks(remaining_tasks);
    }

    /// Функция создает следующую фазу состояния, где задача `task` добавлена в план.
    ///
    /// Функция не проверяет, возможно ли добавить задачу в план.
    /// Перед ее вызовом необходимо убедиться, что в слоте достаточно времени, чтобы задача могла
    /// быть запланирована.
    ///
    pub(super) fn create_next_state(&self, task_ref: TaskRef<'a, Unscheduled>) -> Self {
        let task = task_ref.task();
        let priority = u64::from(*task.priority());

        let mut table = self.table.clone();
        table
            .get_mut(&task.estimated_duration())
            .expect("Задача должна быть представлена в таблице")
            .remove(&task_ref);

        let current_slot = self.current_slot.expect("Current slot should be set");
        let plan = self.plan.clone().with_task(
            task_ref.record_id().clone(),
            current_slot.id().clone(),
            self.now,
            priority,
        );

        Self {
            plan,
            table,
            now: self.now + task.estimated_duration(),
            slots: self.slots.clone(),
            current_slot: self.current_slot,
        }
    }

    pub(super) fn next_from_duration(&mut self, duration: TimeDelta) -> Self {
        let mut next = self.clone();

        let task_ref = next
            .table
            .get_mut(&duration)
            .expect("Строка по ключу duration должна существовать")
            .pop_last()
            .expect("Строка не может быть пустой");

        let task = task_ref.task();
        let priority = u64::from(*task.priority());
        let current_slot = self.current_slot.expect("Текущий слот должен быть задан");

        next.plan.add_task(
            task_ref.record_id().clone(),
            current_slot.id().clone(),
            self.now,
            priority,
        );

        next.now += duration;

        next
    }

    /// Метод ищет первый слот, в котором будет достаточно времени,
    /// чтобы выполнить самую короткую задачу. Слоты до найденного будут удалены из списка.
    ///
    /// Метод обновляет поле ``now`` - она задает его равным максимуму из ``now`` и времени
    /// начала слота. Если слота не нашлось, ``now`` не обновляется.
    ///
    /// ## Возвращаемое значение
    /// * Метод вернет ``None``, если подходящего слота не нашлось
    ///   (в том числе если очередь слотов пуста) или список задач пуст.
    ///   При этом список слотов в состоянии будет очищен.
    ///
    /// * Метод вернет ``Some(time_delta)``, если подходящий слот найдется.
    ///   Значение ``time_delta`` будет равняться оставшемуся в слоте времени.
    ///
    #[must_use]
    pub(super) fn get_available_time(&mut self) -> Option<TimeDelta> {
        self.skip_unsuitable_slots();

        self.slots.front().copied().map(|slot| {
            self.now = cmp::max(self.now, slot.starts_at());
            self.current_slot = Some(slot);

            slot.ends_at() - self.now
        })
    }

    /// Метод удаляет прошедшие ведущие слоты в списке или те,
    /// в которых осталось времени меньше, чем ``min_duration``.
    ///
    fn skip_unsuitable_slots(&mut self) {
        let count = self
            .slots
            .iter()
            .copied()
            .position(|slot| {
                // slot.ends_at() - latest >= min_duration
                let applicable_tasks = self.table.values().flatten().copied().filter(|task_ref| {
                    let task = task_ref.task();
                    let latest = cmp::max(self.now, slot.starts_at());
                    let available_time = slot.ends_at() - latest;
                    task.estimated_duration() <= available_time
                        && task.deadline_as_datetime() >= latest + task.estimated_duration()
                });

                applicable_tasks.count() > 0
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
    /// Если после работы этого метода какие-то строки таблицы остались пустыми,
    /// они удаляются.
    ///
    /// Удаленные задачи добавляются в список отклоненных.
    ///
    pub(super) fn discard_overdue_tasks(&mut self) {
        self.table.values_mut().for_each(|task_set| {
            let overdue_tasks: BTreeSet<TaskRef<'a, Unscheduled>> = task_set
                .iter()
                .filter(|task_ref| {
                    let task = task_ref.task();
                    task.deadline_as_datetime() < self.now + task.estimated_duration()
                })
                .copied()
                .collect();

            self.plan
                .discard_tasks(overdue_tasks.iter().map(|tr| tr.record_id().clone()));
            *task_set = task_set.difference(&overdue_tasks).copied().collect()
        });

        self.remove_empty_rows();
    }

    /// Метод строит таблицу, которая группирует задачи по отведенному на них времени.
    ///
    fn construct_duration_table(
        tasks: &'a [Task<Unscheduled>],
    ) -> BTreeMap<TimeDelta, BTreeSet<TaskRef<'a, Unscheduled>>> {
        tasks.iter().fold(BTreeMap::new(), |mut table, task| {
            let task_ref = TaskRef::new(task);
            table
                .entry(task_ref.task().estimated_duration())
                .or_default()
                .insert(task_ref);
            table
        })
    }

    pub(super) fn table(&self) -> &BTreeMap<TimeDelta, BTreeSet<TaskRef<'a, Unscheduled>>> {
        &self.table
    }

    pub(super) fn table_mut(
        &mut self,
    ) -> &mut BTreeMap<TimeDelta, BTreeSet<TaskRef<'a, Unscheduled>>> {
        &mut self.table
    }

    pub(super) fn plan(&self) -> &Plan {
        &self.plan
    }

    pub(super) fn take_plan(self) -> Plan {
        self.plan
    }

    pub(super) fn slots(&self) -> &VecDeque<&'a Slot> {
        &self.slots
    }
}

#[cfg(test)]
mod tests;
