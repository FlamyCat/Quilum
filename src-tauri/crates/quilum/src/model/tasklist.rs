use std::collections::HashSet;

pub(crate) struct TaskList {
    pub(crate) tasks: HashSet<crate::model::task::Task>,
    pub(crate) title: String,
}

impl TaskList {
    pub(crate) fn new(title: String) -> Self {
        Self {
            title,
            tasks: HashSet::new(),
        }
    }
}

pub(crate) struct TaskListRecord {
    pub(crate) title: String
}
