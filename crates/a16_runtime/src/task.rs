//! Task definitions for the async runtime

use smol_str::SmolStr;

/// Unique task identifier
pub type TaskId = u64;

/// Task scheduling priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// Normal priority (default)
    Normal = 0,
    /// High priority (scheduled before normal)
    High = 1,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// Result of a completed task
#[derive(Debug, Clone)]
pub enum TaskResult {
    /// Task completed successfully with a value
    Value(SmolStr),
    /// Task completed with a numeric result (for VM integration)
    ValueId(u64),
    /// Task completed with an error
    Error(SmolStr),
    /// Task was cancelled
    Cancelled,
}

/// Task execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Ready to execute
    Ready,
    /// Waiting on I/O or another task
    Waiting,
    /// Currently executing
    Running,
    /// Completed (successfully or not)
    Done,
    /// Cancelled before completion
    Cancelled,
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskState::Ready => write!(f, "Ready"),
            TaskState::Waiting => write!(f, "Waiting"),
            TaskState::Running => write!(f, "Running"),
            TaskState::Done => write!(f, "Done"),
            TaskState::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// An async task in the runtime
#[derive(Debug)]
pub struct Task {
    /// Task identifier
    pub id: TaskId,
    /// Human-readable name
    pub name: SmolStr,
    /// Current state
    pub state: TaskState,
    /// Function index to execute (in bytecode module)
    pub func_idx: u16,
    /// Instruction pointer (suspended position)
    pub ip: usize,
    /// Task-local stack
    pub stack: Vec<SmolStr>,
    /// Result when completed
    pub result: Option<TaskResult>,
    /// ID of task this is waiting on (if any)
    pub waiting_on: Option<TaskId>,
    /// Task priority
    pub priority: TaskPriority,
}

impl Task {
    /// Create a new ready task
    pub fn new(id: TaskId, name: SmolStr, func_idx: u16) -> Self {
        Self {
            id,
            name,
            state: TaskState::Ready,
            func_idx,
            ip: 0,
            stack: Vec::new(),
            result: None,
            waiting_on: None,
            priority: TaskPriority::Normal,
        }
    }

    /// Create a new task with priority
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Check if task is done
    pub fn is_done(&self) -> bool {
        matches!(self.state, TaskState::Done | TaskState::Cancelled)
    }

    /// Complete the task with a result
    pub fn complete(&mut self, result: TaskResult) {
        self.state = TaskState::Done;
        self.result = Some(result);
    }

    /// Cancel the task
    pub fn cancel(&mut self) {
        self.state = TaskState::Cancelled;
        self.result = Some(TaskResult::Cancelled);
    }

    /// Set waiting state
    pub fn wait_on(&mut self, target: TaskId) {
        self.state = TaskState::Waiting;
        self.waiting_on = Some(target);
    }

    /// Resume from waiting
    pub fn resume(&mut self) {
        self.state = TaskState::Ready;
        self.waiting_on = None;
    }
}
