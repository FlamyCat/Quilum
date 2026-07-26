use chrono::{NaiveDate, TimeDelta};
use surrealdb::Error;

use crate::{
    SlotWithTasks, Storage,
    event::EVENTS_TABLE,
    model::{event::Event, task::Task},
};

impl Storage {
    /// Gets events overlapping a date range (inclusive start, exclusive end).
    ///
    /// # Arguments
    /// * `start` - The start date (inclusive, at 00:00:00)
    /// * `end` - The end date (exclusive, at 00:00:00)
    ///
    /// # Returns
    /// * Vector of events occurring in the date range
    pub async fn get_events_for_date_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Event>, Error> {
        let range_start = start.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let range_end = end.and_hms_opt(0, 0, 0).unwrap().and_utc();

        let sql = format!(
            "SELECT * FROM {} WHERE starts_at IN $start..$end OR ends_at IN $start..$end",
            EVENTS_TABLE
        );

        let mut result = self
            .db
            .query(sql)
            .bind(("start", range_start))
            .bind(("end", range_end))
            .await?;

        result.take(0)
    }

    /// Gets events occurring on a specific date.
    /// Events that overlap with the date are returned (even if they span multiple days).
    ///
    /// # Arguments
    /// * `date` - The date to query
    ///
    /// # Returns
    /// * Vector of events occurring on the date
    pub async fn get_events_for_date(&self, date: NaiveDate) -> Result<Vec<Event>, Error> {
        let next_day = date + TimeDelta::days(1);
        self.get_events_for_date_range(date, next_day).await
    }
}

impl Storage {
    /// Gets all tasks scheduled in slots within a date range.
    /// Returns flat list of tasks with their scheduled_for timestamps (for today view).
    ///
    /// # Arguments
    /// * `start` - The start date (inclusive, at 00:00:00)
    /// * `end` - The end date (exclusive, at 00:00:00)
    ///
    /// # Returns
    /// * Vector of tasks with their scheduled_for timestamps
    pub async fn get_scheduled_tasks_for_date_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Task>, Error> {
        let range_start = start.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let range_end = end.and_hms_opt(0, 0, 0).unwrap().and_utc();

        let sql = "
            SELECT *
            FROM tasks
            WHERE
                scheduled_for != NONE
                AND scheduled_for IN $start..$end
        ";

        self.db
            .query(sql)
            .bind(("start", range_start))
            .bind(("end", range_end))
            .await?
            .take(0)
    }

    /// Gets the next scheduled task that hasn't ended yet.
    /// Returns the scheduled task.
    ///
    /// This returns tasks that are either:
    /// - Currently in progress (`now in scheduled_for..(scheduled_for + duration)`)
    /// - Scheduled to start in the future (`scheduled_for > now`)
    ///
    /// # Returns
    /// * Scheduled task if found
    pub async fn get_next_scheduled_task(&self) -> Result<Option<Task>, Error> {
        let sql = "
            SELECT *
            FROM ONLY tasks
            WHERE
                scheduled_for + estimated_duration >= time::now()
                AND completed == false
            ORDER BY scheduled_for
            LIMIT 1;
        ";

        self.db.query(sql).await?.take(0)
    }

    /// Gets all slots within a date range along with their scheduled tasks.
    /// Returns ALL slots (including empty ones) grouped with their tasks.
    ///
    /// # Arguments
    /// * `start` - The start date (inclusive, at 00:00:00)
    /// * `end` - The end date (exclusive, at 00:00:00)
    ///
    /// # Returns
    /// * Vector of slots with their scheduled tasks
    pub async fn get_slots_with_tasks_for_date_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<SlotWithTasks>, Error> {
        let sql = "
            SELECT
                *,
                <-scheduled_in<-tasks AS tasks
            FROM slots
            WHERE
                starts_at IN $start..$end
                OR ends_at IN $start..$end
            FETCH tasks
        ";

        self.db
            .query(sql)
            .bind((
                "start",
                start.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc(),
            ))
            .bind((
                "end",
                end.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc(),
            ))
            .await?
            .take(0)
    }

    /// Gets today's timetable: events + scheduled tasks for today view.
    ///
    /// # Arguments
    /// * `today` - The date to get timetable for
    ///
    /// # Returns
    /// * Tuple of (events, scheduled_tasks) for today
    pub async fn get_today_timetable(
        &self,
        today: NaiveDate,
    ) -> Result<(Vec<Event>, Vec<Task>), Error> {
        let tomorrow = today + TimeDelta::days(1);

        let events = self.get_events_for_date(today).await?;
        let scheduled_tasks = self
            .get_scheduled_tasks_for_date_range(today, tomorrow)
            .await?;

        Ok((events, scheduled_tasks))
    }

    /// Gets week's timetable: events + slots with tasks for week view.
    /// Derives week_end as week_start + 7 days (half-open range).
    ///
    /// # Arguments
    /// * `week_start` - The first day of the week
    ///
    /// # Returns
    /// * Tuple of (events, slots_with_tasks) for the week
    pub async fn get_week_timetable(
        &self,
        week_start: NaiveDate,
    ) -> Result<(Vec<Event>, Vec<SlotWithTasks>), Error> {
        let week_end = week_start + TimeDelta::days(7);

        let events = self.get_events_for_date_range(week_start, week_end).await?;
        let slots_with_tasks = self
            .get_slots_with_tasks_for_date_range(week_start, week_end)
            .await?;

        Ok((events, slots_with_tasks))
    }
}

#[cfg(test)]
mod tests;
