use crate::{SlotWithTasks, Storage, event::Event, slot::Slot, task::Task};
use chrono::{NaiveDate, TimeDelta};
use surrealdb::{Error, types::RecordId};

impl Storage {
    /// Gets events occurring within a date range (inclusive start, exclusive end).
    /// Events that overlap with the date range are returned (even if they span multiple days).
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
        let range_start = start.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let range_end = end.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

        let sql = format!(
            "SELECT * FROM event WHERE starts_at < {} AND ends_at > {}",
            range_end, range_start
        );
        let mut result = self.db.query(sql).await?;
        let events: Vec<Event> = result.take(0).unwrap_or_default();
        Ok(events)
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
    ) -> Result<Vec<(Task, i64)>, Error> {
        let range_start = start.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let range_end = end.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

        // Use graph traversal to get task data directly
        let sql = format!(
            "SELECT out.*, scheduled_for FROM contains \
            WHERE in.starts_at < {} AND in.ends_at > {}",
            range_end, range_start
        );
        let mut result = self.db.query(sql).await?;
        let raw: Vec<serde_json::Value> = result.take(0).unwrap_or_default();

        let mut scheduled_tasks = Vec::new();
        for item in raw {
            if let (Some(task_value), Some(scheduled_for)) =
                (item.get("out"), item.get("scheduled_for"))
            {
                if let (Some(task_obj), Some(sf)) = (task_value.as_object(), scheduled_for.as_i64())
                {
                    // Convert the task object to proper JSON, handling id field
                    let mut task_json = serde_json::Map::new();
                    for (k, v) in task_obj {
                        if k == "id" {
                            // id is a string like "task:xxx", convert to object format for RecordId
                            if let Some(id_str) = v.as_str() {
                                let parts: Vec<&str> = id_str.split(':').collect();
                                if parts.len() == 2 {
                                    let mut id_obj = serde_json::Map::new();
                                    id_obj.insert(
                                        "table".to_string(),
                                        serde_json::Value::String(parts[0].to_string()),
                                    );
                                    id_obj.insert(
                                        "key".to_string(),
                                        serde_json::json!({"String": parts[1]}),
                                    );
                                    task_json.insert(
                                        "id".to_string(),
                                        serde_json::Value::Object(id_obj),
                                    );
                                }
                            }
                        } else if k == "priority" {
                            // Handle priority enum - extract the variant name
                            if let Some(priority_obj) = v.as_object() {
                                if let Some(first_key) = priority_obj.keys().next() {
                                    task_json.insert(
                                        "priority".to_string(),
                                        serde_json::Value::String(first_key.clone()),
                                    );
                                }
                            }
                        } else {
                            task_json.insert(k.clone(), v.clone());
                        }
                    }
                    let task: Task = serde_json::from_value(serde_json::Value::Object(task_json))
                        .map_err(|e| {
                        Error::query(format!("Failed to deserialize task: {}", e), None)
                    })?;
                    scheduled_tasks.push((task, sf));
                }
            }
        }
        Ok(scheduled_tasks)
    }

    /// Gets the next scheduled task that hasn't ended yet.
    /// Returns the task and its scheduled_for timestamp.
    ///
    /// This returns tasks that are either:
    /// - Currently in progress (scheduled_for <= now <= scheduled_for + duration)
    /// - Scheduled to start in the future (scheduled_for > now)
    ///
    /// # Returns
    /// * Option containing (Task, scheduled_for_timestamp) if found
    pub async fn get_next_scheduled_task(&self) -> Result<Option<(Task, i64)>, Error> {
        let now = chrono::Utc::now().timestamp();
        let sql = format!(
            "SELECT scheduled_for, out.* AS task FROM ONLY contains \
             WHERE scheduled_for + out.estimated_duration >= {} \
             ORDER BY scheduled_for \
             LIMIT 1",
            now
        );
        let mut result = self.db.query(sql).await?;
        let value: Option<serde_json::Value> = result.take(0).unwrap_or_default();

        let Some(item) = value else {
            return Ok(None);
        };

        let scheduled_for = item
            .get("scheduled_for")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| Error::query("Missing scheduled_for".to_string(), None))?;

        let task_value = item
            .get("task")
            .ok_or_else(|| Error::query("Missing task".to_string(), None))?;

        let mut task_json = serde_json::Map::new();
        if let Some(obj) = task_value.as_object() {
            for (k, v) in obj {
                if k == "id" {
                    if let Some(id_str) = v.as_str() {
                        let parts: Vec<&str> = id_str.split(':').collect();
                        if parts.len() == 2 {
                            let mut id_obj = serde_json::Map::new();
                            id_obj.insert(
                                "table".to_string(),
                                serde_json::Value::String(parts[0].to_string()),
                            );
                            id_obj
                                .insert("key".to_string(), serde_json::json!({"String": parts[1]}));
                            task_json.insert("id".to_string(), serde_json::Value::Object(id_obj));
                        }
                    }
                } else if k == "priority" {
                    if let Some(priority_obj) = v.as_object()
                        && let Some(first_key) = priority_obj.keys().next()
                    {
                        task_json.insert(
                            "priority".to_string(),
                            serde_json::Value::String(first_key.clone()),
                        );
                    }
                } else {
                    task_json.insert(k.clone(), v.clone());
                }
            }
        }

        let task: Task = serde_json::from_value(serde_json::Value::Object(task_json))
            .map_err(|e| Error::query(format!("Failed to deserialize task: {}", e), None))?;

        if scheduled_for + task.estimated_duration <= now {
            return Ok(None);
        }

        Ok(Some((task, scheduled_for)))
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
        let range_start = start.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let range_end = end.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

        // Query using graph syntax to get slots + their tasks in one query
        let sql = format!(
            "SELECT *, ->(SELECT out.*, scheduled_for FROM contains) AS tasks \
             FROM slot WHERE starts_at < {} AND ends_at > {}",
            range_end, range_start
        );
        let mut result = self.db.query(sql).await?;
        let slot_values: Vec<serde_json::Value> = result.take(0).unwrap_or_default();

        let mut slots_with_tasks: Vec<SlotWithTasks> = Vec::new();

        for mut slot_value in slot_values {
            // === Parse slot ===
            let _slot_id = {
                let id_str = slot_value
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let parts: Vec<&str> = id_str.split(':').collect();
                if parts.len() == 2 {
                    RecordId::new(parts[0], parts[1])
                } else {
                    continue;
                }
            };

            // Clone tasks BEFORE removing from slot_value
            let tasks_value = slot_value.get("tasks").cloned().unwrap_or_default();

            // Remove non-Slot fields (but keep id for deserialization)
            if let Some(obj) = slot_value.as_object_mut() {
                obj.remove("tasks");
            }

            // Convert the slot object to proper JSON, handling id field
            let mut slot_json = serde_json::Map::new();
            if let Some(obj) = slot_value.as_object() {
                for (k, v) in obj {
                    if k == "id" {
                        // id is a string like "slot:xxx", convert to object format for RecordId
                        if let Some(id_str) = v.as_str() {
                            let parts: Vec<&str> = id_str.split(':').collect();
                            if parts.len() == 2 {
                                let mut id_obj = serde_json::Map::new();
                                id_obj.insert(
                                    "table".to_string(),
                                    serde_json::Value::String(parts[0].to_string()),
                                );
                                id_obj.insert(
                                    "key".to_string(),
                                    serde_json::json!({"String": parts[1]}),
                                );
                                slot_json
                                    .insert("id".to_string(), serde_json::Value::Object(id_obj));
                            }
                        }
                    } else {
                        slot_json.insert(k.clone(), v.clone());
                    }
                }
            }

            let slot: Slot = serde_json::from_value(serde_json::Value::Object(slot_json))
                .map_err(|e| Error::query(format!("Failed to deserialize slot: {}", e), None))?;

            // === Parse tasks ===
            let tasks: Vec<(Task, i64)> = if let serde_json::Value::Array(arr) = tasks_value {
                arr.into_iter()
                    .filter_map(|task_item| {
                        // task_item = {"out": {...}, "scheduled_for": 123...}

                        // Extract scheduled_for from top level
                        let scheduled_for =
                            task_item.get("scheduled_for").and_then(|v| v.as_i64())?;

                        // Extract "out" object which contains task data
                        let out_value = task_item.get("out").cloned()?;
                        let out_obj = if let serde_json::Value::Object(obj) = out_value {
                            obj
                        } else {
                            return None;
                        };

                        // Convert the task object to proper JSON, handling id field
                        let mut task_json = serde_json::Map::new();
                        for (k, v) in &out_obj {
                            if k == "id" {
                                // id is a string like "task:xxx", convert to object format for RecordId
                                if let Some(id_str) = v.as_str() {
                                    let parts: Vec<&str> = id_str.split(':').collect();
                                    if parts.len() == 2 {
                                        let mut id_obj = serde_json::Map::new();
                                        id_obj.insert(
                                            "table".to_string(),
                                            serde_json::Value::String(parts[0].to_string()),
                                        );
                                        id_obj.insert(
                                            "key".to_string(),
                                            serde_json::json!({"String": parts[1]}),
                                        );
                                        task_json.insert(
                                            "id".to_string(),
                                            serde_json::Value::Object(id_obj),
                                        );
                                    }
                                }
                            } else if k == "priority" {
                                // Handle priority enum - extract the variant name
                                if let Some(priority_obj) = v.as_object() {
                                    if let Some(first_key) = priority_obj.keys().next() {
                                        task_json.insert(
                                            "priority".to_string(),
                                            serde_json::Value::String(first_key.clone()),
                                        );
                                    }
                                }
                            } else {
                                task_json.insert(k.clone(), v.clone());
                            }
                        }

                        let task: Task =
                            serde_json::from_value(serde_json::Value::Object(task_json)).ok()?;

                        Some((task, scheduled_for))
                    })
                    .collect()
            } else {
                vec![]
            };

            slots_with_tasks.push(SlotWithTasks { slot, tasks });
        }

        Ok(slots_with_tasks)
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
    ) -> Result<(Vec<Event>, Vec<(Task, i64)>), Error> {
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
