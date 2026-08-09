use chrono::{DateTime, FixedOffset, NaiveDate, TimeDelta, Utc};
use surrealdb::Error;

use crate::{
    SlotWithTasks, Storage,
    event::EVENTS_TABLE,
    model::{event::Event, task::Task},
};

/// Converts a calendar date into the UTC instant of its local midnight,
/// using the given timezone offset. This makes "a date" mean "that date's
/// local day", so day ranges cover the whole local day regardless of the
/// machine's timezone.
fn local_midnight_utc(date: NaiveDate, offset: FixedOffset) -> DateTime<Utc> {
    date.and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(offset)
        .unwrap()
        .with_timezone(&Utc)
}

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
        offset: FixedOffset,
    ) -> Result<Vec<Event>, Error> {
        let range_start = local_midnight_utc(start, offset);
        let range_end = local_midnight_utc(end, offset);

        let sql = format!(
            "SELECT * FROM {} \
            WHERE starts_at < $end AND ends_at > $start",
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
    pub async fn get_events_for_date(
        &self,
        date: NaiveDate,
        offset: FixedOffset,
    ) -> Result<Vec<Event>, Error> {
        let next_day = date + TimeDelta::days(1);
        self.get_events_for_date_range(date, next_day, offset).await
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
        offset: FixedOffset,
    ) -> Result<Vec<Task>, Error> {
        let range_start = local_midnight_utc(start, offset);
        let range_end = local_midnight_utc(end, offset);

        let sql = "
            SELECT *
            FROM tasks
            WHERE
                scheduled_for != NONE
                AND scheduled_for + estimated_duration >= $start
                AND scheduled_for < $end
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
        offset: FixedOffset,
    ) -> Result<Vec<SlotWithTasks>, Error> {
        let sql = "
            SELECT
               {
                   id: id,
                   starts_at: starts_at,
                   ends_at: ends_at,
               } AS slot,
                <-scheduled_in<-tasks AS tasks
            FROM slots
            WHERE
                starts_at < $end AND ends_at > $start
            FETCH tasks
        ";

        self.db
            .query(sql)
            .bind(("start", local_midnight_utc(start, offset)))
            .bind(("end", local_midnight_utc(end, offset)))
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
        offset: FixedOffset,
    ) -> Result<(Vec<Event>, Vec<Task>), Error> {
        let tomorrow = today + TimeDelta::days(1);

        let events = self.get_events_for_date(today, offset).await?;
        let scheduled_tasks = self
            .get_scheduled_tasks_for_date_range(today, tomorrow, offset)
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
        offset: FixedOffset,
    ) -> Result<(Vec<Event>, Vec<SlotWithTasks>), Error> {
        let week_end = week_start + TimeDelta::days(7);

        let events = self
            .get_events_for_date_range(week_start, week_end, offset)
            .await?;
        let slots_with_tasks = self
            .get_slots_with_tasks_for_date_range(week_start, week_end, offset)
            .await?;

        Ok((events, slots_with_tasks))
    }
}

#[cfg(test)]
mod tests;
