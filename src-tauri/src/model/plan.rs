use crate::model::task::{ScheduledTask, Task};
use std::ops::Index;

#[derive(Clone, Debug)]
pub(crate) struct Plan<'a> {
    tasks: Vec<ScheduledTask<'a>>,
    discarded_tasks: Vec<&'a Task>,
    score: u64,
}

impl<'a> Index<usize> for Plan<'a> {
    type Output = ScheduledTask<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.tasks[index]
    }
}

impl<'a> Plan<'a> {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            discarded_tasks: Vec::new(),
            score: 0,
        }
    }

    pub fn add_task(&mut self, task: ScheduledTask<'a>) {
        self.tasks.push(task);
        self.score += u64::from(task.priority());
    }

    pub fn with_task(self, task: ScheduledTask<'a>) -> Self {
        let mut tasks = self.tasks;
        tasks.push(task);

        Self {
            score: self.score + u64::from(task.priority()),
            tasks,
            ..self
        }
    }

    pub fn discard_task(&mut self, task: &'a Task) {
        self.discarded_tasks.push(task);
    }

    pub fn discard_tasks(&mut self, tasks: impl Iterator<Item = &'a Task>) {
        tasks.collect_into(&mut self.discarded_tasks);
    }

    pub fn score(&self) -> u64 {
        self.score
    }

    pub fn tasks(&self) -> &Vec<ScheduledTask<'a>> {
        &self.tasks
    }

    pub fn discarded_tasks(&self) -> &Vec<&'a Task> {
        &self.discarded_tasks
    }
}
