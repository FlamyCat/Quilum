mod state;

use crate::db::{Record, Storage};
use crate::model::{plan::Plan, slot::Slot, task::Task};
use chrono::{Local, NaiveDateTime};
use state::State;

pub(crate) struct Scheduler<'a> {
    tasks: &'a [Record<Task>],
    upcoming_slots: &'a [Record<Slot>],
    now: NaiveDateTime,
    storage: &'a Storage,
}

impl<'a> Scheduler<'a> {
    pub fn new(tasks: &'a [Record<Task>], upcoming_slots: &'a [Record<Slot>], now: NaiveDateTime, storage: &'a Storage) -> Self {
        Self {
            tasks,
            upcoming_slots,
            now,
            storage,
        }
    }

    pub fn new_with_local_datetime(tasks: &'a [Record<Task>], upcoming_slots: &'a [Record<Slot>], storage: &'a Storage) -> Self {
        Self::new(tasks, upcoming_slots, Local::now().naive_local(), storage)
    }

    /// Метод составляет план на основе полученных задач
    ///
    pub(crate) fn schedule(&self) -> Plan {
        let state = State::new(self.tasks, self.upcoming_slots.iter().collect(), self.now);

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

        let Some(remaining_time) = state.get_available_time() else {
            state.discard_remaining_tasks();

            return state.take_plan();
        };

        state.discard_overdue_tasks();

        let durations: Vec<_> = state
            .table()
            .keys()
            .filter(|&&duration| duration <= remaining_time)
            .copied()
            .collect();

        durations
            .into_iter()
            .map(|duration| state.next_from_duration(duration))
            .map(Self::schedule_from)
            .max_by_key(Plan::score)
            .unwrap()
    }

    /// Составляет план и создает связи в базе данных
    ///
    pub(crate) async fn schedule_and_commit(&self) -> Result<Plan, surrealdb::Error> {
        let plan = self.schedule();

        for (task_id, slot_id, scheduled_for) in plan.tasks() {
            self.storage.relate_task_to_slot(slot_id, task_id, *scheduled_for).await?;
        }

        Ok(plan)
    }
}

#[cfg(test)]
mod tests;
