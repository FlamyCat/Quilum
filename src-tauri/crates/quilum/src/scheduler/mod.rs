mod state;

use crate::model::{plan::Plan, slot::Slot, task::Task};
use chrono::{Local, NaiveDateTime};
use state::State;

pub(crate) struct Scheduler<'a> {
    tasks: &'a [Task],
    upcoming_slots: &'a [Slot],
    now: NaiveDateTime,
}

impl<'a> Scheduler<'a> {
    pub fn new(tasks: &'a [Task], upcoming_slots: &'a [Slot], now: NaiveDateTime) -> Self {
        Self {
            tasks,
            upcoming_slots,
            now,
        }
    }

    pub fn new_with_local_datetime(tasks: &'a [Task], upcoming_slots: &'a [Slot]) -> Self {
        Self::new(tasks, upcoming_slots, Local::now().naive_local())
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

        state
            .table()
            .keys()
            .filter_map(|&duration| {
                if duration <= remaining_time {
                    Some(state.next_from_duration(duration))
                } else {
                    None
                }
            })
            .map(Self::schedule_from)
            .max_by_key(Plan::score)
            .unwrap()
    }
}

#[cfg(test)]
mod tests;
