use std::{cmp::Ordering, fmt::Debug, marker::PhantomData};

use chrono::{DateTime, NaiveDateTime, TimeDelta};
use serde::{Deserialize, Serialize};
use surrealdb::{
    types::{Kind, RecordId, RecordIdKey, SurrealValue, Value},
    Error,
};
use thiserror::Error;

/// Marks the task as scheduled.
/// For [`Task`] instances of this state it is safe to access scheduling information.
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Scheduled;

/// Marks the task as unscheduled.
///
/// This state **DOES NOT** necessarily represent the state of the storage, it just means that the
/// current task instance does not carry the scheduling information.
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Unscheduled;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task<S> {
    data: TaskData,
    _schedule_info: PhantomData<S>,
}

impl SurrealValue for Task<Scheduled> {
    fn kind_of() -> Kind {
        TaskData::kind_of()
    }

    fn into_value(self) -> Value {
        self.data.into_value()
    }

    fn from_value(value: Value) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let data = TaskData::from_value(value)?;
        Task::try_scheduled_from_data(data).map_err(|e| Error::serialization(e.to_string(), None))
    }
}

impl SurrealValue for Task<Unscheduled> {
    fn kind_of() -> Kind {
        TaskData::kind_of()
    }

    fn into_value(self) -> Value {
        self.data.into_value()
    }

    fn from_value(value: Value) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let data = TaskData::from_value(value)?;
        Task::try_unscheduled_from_data(data).map_err(|e| Error::serialization(e.to_string(), None))
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, SurrealValue)]
pub struct TaskData {
    pub id: RecordId,
    pub name: String,
    pub description: String,
    pub priority: Priority,
    pub estimated_duration: i64,
    pub deadline: i64,
    pub completed: bool,
    pub scheduled_for: Option<i64>,
}

pub const TASKS_TABLE: &str = "tasks";

impl TaskData {
    pub fn new(
        name: String,
        description: String,
        priority: Priority,
        estimated_duration: i64,
        deadline: i64,
        completed: bool,
        scheduled_for: Option<i64>,
    ) -> Self {
        Self {
            id: RecordId::new(TASKS_TABLE, RecordIdKey::ulid()),
            name,
            description,
            priority,
            estimated_duration,
            deadline,
            completed,
            scheduled_for,
        }
    }
}

enum TaskVariant {
    Scheduled(Task<Scheduled>),
    Unscheduled(Task<Unscheduled>),
}

#[derive(Error, Debug)]
pub enum TaskSerializationError {
    #[error("Failed to deserialize a with a wrong scheduling kind: {0}")]
    WrongSchedulingKind(String),
}

impl Task<Unscheduled> {
    /// Constructs a new **unscheduled** task from given `data`.
    /// If `scheduled_for` in `data` is `Some`, the method will panic.
    pub fn unscheduled_from_data(data: TaskData) -> Self {
        assert!(
            data.scheduled_for.is_none(),
            "Attempted constructing an unscheduled task from scheduled task data"
        );

        Self {
            data,
            _schedule_info: PhantomData,
        }
    }

    /// Constructs a new **unscheduled** task from given `data`.
    /// If `scheduled_for` in `data` is `Some`, the method will return an error.
    pub fn try_unscheduled_from_data(data: TaskData) -> Result<Self, TaskSerializationError> {
        if data.scheduled_for.is_some() {
            Err(TaskSerializationError::WrongSchedulingKind(String::from(
                "value being deserialized did carry the scheduling data",
            )))
        } else {
            Ok(Self::unscheduled_from_data(data))
        }
    }
}

impl Task<Scheduled> {
    /// Constructs a new **scheduled** task from given `data`.
    /// If `scheduled_for` in `data` is `None`, the method will panic.
    pub fn scheduled_from_data(data: TaskData) -> Self {
        assert!(
            data.scheduled_for.is_none(),
            "Attempted constructing a scheduled task from unscheduled task data"
        );

        Self {
            data,
            _schedule_info: PhantomData,
        }
    }

    /// Constructs a new **scheduled** task from given `data`.
    /// If `scheduled_for` in `data` is `None`, the method will return an error.
    pub fn try_scheduled_from_data(data: TaskData) -> Result<Self, TaskSerializationError> {
        if data.scheduled_for.is_some() {
            Err(TaskSerializationError::WrongSchedulingKind(String::from(
                "value being deserialized did not carry the scheduling data",
            )))
        } else {
            Ok(Self::scheduled_from_data(data))
        }
    }
}

impl<S: Ord> PartialOrd for Task<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: Ord> Ord for Task<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        fn to_priority_tuple<S>(task: &Task<S>) -> (u64, i64, &String, &String) {
            (
                u64::from(task.data.priority),
                task.data.deadline,
                &task.data.name,
                &task.data.description,
            )
        }

        to_priority_tuple(self).cmp(&to_priority_tuple(other))
    }
}

impl<S> Task<S> {
    pub fn name(&self) -> &str {
        &self.data.name
    }

    pub fn description(&self) -> &str {
        &self.data.description
    }

    pub fn priority(&self) -> &Priority {
        &self.data.priority
    }

    pub fn deadline(&self) -> i64 {
        self.data.deadline
    }

    pub fn deadline_as_datetime(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.data.deadline, 0)
            .unwrap_or_default()
            .naive_utc()
    }

    pub fn id(&self) -> &RecordId {
        &self.data.id
    }

    pub fn estimated_duration(&self) -> TimeDelta {
        TimeDelta::seconds(self.data.estimated_duration)
    }

    pub fn deadline_datetime(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.data.deadline, 0)
            .unwrap_or_default()
            .naive_utc()
    }

    pub fn set_name(&mut self, name: String) {
        self.data.name = name;
    }

    pub fn set_description(&mut self, description: String) {
        self.data.description = description;
    }

    pub fn set_priority(&mut self, priority: Priority) {
        self.data.priority = priority;
    }

    pub fn set_estimated_duration(&mut self, estimated_duration: TimeDelta) {
        self.data.estimated_duration = estimated_duration.num_seconds();
    }

    pub fn set_deadline(&mut self, deadline: NaiveDateTime) {
        self.data.deadline = deadline.and_utc().timestamp();
    }

    pub fn completed(&self) -> bool {
        self.data.completed
    }

    pub fn set_completed(&mut self, completed: bool) {
        self.data.completed = completed;
    }

    /// Transforms the task into scheduled one, with `scheduled_for` set to `timestamp`.
    pub fn schedule_for(self, timestamp: i64) -> Task<Scheduled> {
        Task::<Scheduled> {
            data: TaskData {
                scheduled_for: Some(timestamp),
                ..self.data
            },
            _schedule_info: PhantomData,
        }
    }
}

impl Task<Scheduled> {
    pub fn scheduled_for(&self) -> i64 {
        self.data.scheduled_for.unwrap()
    }
}

#[derive(
    Copy, Clone, Debug, Hash, Eq, PartialEq, Default, Serialize, Deserialize, SurrealValue,
)]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
}

impl From<Priority> for u64 {
    fn from(value: Priority) -> Self {
        let priority_as_number: u64 = match value {
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
        };

        priority_as_number.pow(2)
    }
}

impl From<&Priority> for u64 {
    fn from(value: &Priority) -> Self {
        u64::from(*value)
    }
}
