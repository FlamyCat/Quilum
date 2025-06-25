use crate::model::task::ScheduledTask;

#[derive(Clone, Debug)]
pub(crate) struct Plan<'a> {
    tasks: Vec<ScheduledTask<'a>>,
    score: u64,
}

impl<'a> Plan<'a> {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
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
        }
    }
    
    pub fn score(&self) -> u64 {
        self.score
    }
}
