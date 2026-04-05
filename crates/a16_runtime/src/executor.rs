//! Single-threaded cooperative task executor
//!
//! Implements round-robin scheduling with priority support.

use indexmap::IndexMap;
use smol_str::SmolStr;
use thiserror::Error;
use std::collections::VecDeque;

use crate::task::{Task, TaskId, TaskState, TaskResult, TaskPriority};

/// Executor errors
#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Task {0} not found")]
    TaskNotFound(TaskId),

    #[error("No tasks to execute")]
    NoTasks,

    #[error("Deadlock detected: all tasks are waiting")]
    Deadlock,

    #[error("Maximum tick count exceeded")]
    MaxTicksExceeded,
}

/// Single-threaded cooperative executor
#[derive(Debug)]
pub struct Executor {
    /// All registered tasks
    tasks: IndexMap<TaskId, Task>,
    /// Ready queue (FIFO)
    ready_queue: VecDeque<TaskId>,
    /// Next task ID
    next_id: TaskId,
    /// Total ticks executed
    ticks: u64,
    /// Maximum ticks before force-stopping
    max_ticks: u64,
}

impl Executor {
    /// Create a new executor
    pub fn new() -> Self {
        Self {
            tasks: IndexMap::new(),
            ready_queue: VecDeque::new(),
            next_id: 1,
            ticks: 0,
            max_ticks: 100_000,
        }
    }

    /// Set the maximum tick count
    pub fn with_max_ticks(mut self, max: u64) -> Self {
        self.max_ticks = max;
        self
    }

    /// Spawn a new task
    pub fn spawn(&mut self, name: SmolStr, func_idx: u16) -> TaskId {
        let id = self.next_id;
        self.next_id += 1;

        let task = Task::new(id, name, func_idx);
        self.tasks.insert(id, task);
        self.ready_queue.push_back(id);

        id
    }

    /// Spawn a new task with priority
    pub fn spawn_with_priority(&mut self, name: SmolStr, func_idx: u16, priority: TaskPriority) -> TaskId {
        let id = self.next_id;
        self.next_id += 1;

        let task = Task::new(id, name, func_idx).with_priority(priority);
        self.tasks.insert(id, task);

        // High priority tasks go to the front of the queue
        match priority {
            TaskPriority::High => self.ready_queue.push_front(id),
            TaskPriority::Normal => self.ready_queue.push_back(id),
        }

        id
    }

    /// Get a task by ID
    pub fn get_task(&self, id: TaskId) -> Option<&Task> {
        self.tasks.get(&id)
    }

    /// Number of active (non-done) tasks
    pub fn active_count(&self) -> usize {
        self.tasks.values()
            .filter(|t| !t.is_done())
            .count()
    }

    /// Number of completed tasks
    pub fn done_count(&self) -> usize {
        self.tasks.values()
            .filter(|t| t.is_done())
            .count()
    }

    /// Total tasks
    pub fn total_count(&self) -> usize {
        self.tasks.len()
    }

    /// Run a single scheduling tick.
    /// Returns the ID of the task that was stepped, or None if no ready tasks.
    pub fn tick(&mut self) -> Result<Option<TaskId>, ExecutorError> {
        if self.ticks >= self.max_ticks {
            return Err(ExecutorError::MaxTicksExceeded);
        }

        // Check for deadlock
        if self.ready_queue.is_empty() {
            let has_waiting = self.tasks.values().any(|t| t.state == TaskState::Waiting);
            if has_waiting {
                // Check if any waiting tasks can be woken
                self.wake_completed_waiters();
                if self.ready_queue.is_empty() {
                    return Err(ExecutorError::Deadlock);
                }
            } else {
                return Ok(None);
            }
        }

        // Pick the next ready task
        if let Some(task_id) = self.ready_queue.pop_front() {
            if let Some(task) = self.tasks.get_mut(&task_id) {
                if task.state == TaskState::Ready {
                    task.state = TaskState::Running;
                    self.ticks += 1;

                    // Simulate one step of execution
                    // In a real implementation, this would execute bytecode
                    // For now, complete the task immediately
                    task.complete(TaskResult::Value(SmolStr::new("done")));

                    return Ok(Some(task_id));
                }
            }
        }

        Ok(None)
    }

    /// Run all tasks to completion
    pub fn run_all(&mut self) -> Result<(), ExecutorError> {
        while self.active_count() > 0 {
            self.tick()?;
        }
        Ok(())
    }

    /// Check if any task is ready without consuming it
    pub fn poll_any_ready(&self) -> Option<TaskId> {
        self.ready_queue.front().copied()
    }

    /// Wait for a specific set of tasks to complete.
    /// Returns results in the same order as the input IDs.
    pub fn join_all(&mut self, ids: &[TaskId]) -> Result<Vec<Option<TaskResult>>, ExecutorError> {
        // Run until all target tasks are done
        loop {
            let all_done = ids.iter().all(|id| {
                self.tasks.get(id)
                    .map(|t| t.is_done())
                    .unwrap_or(true) // Missing tasks count as done
            });

            if all_done {
                break;
            }

            match self.tick()? {
                Some(_) => continue,
                None => {
                    // No ready tasks and not all done -> deadlock check
                    if self.active_count() == 0 {
                        break;
                    }
                }
            }
        }

        // Collect results in order
        let results = ids.iter().map(|id| {
            self.tasks.get(id).and_then(|t| t.result.clone())
        }).collect();

        Ok(results)
    }

    /// Complete a specific task with a value
    pub fn complete_task(&mut self, id: TaskId, result: TaskResult) -> Result<(), ExecutorError> {
        let task = self.tasks.get_mut(&id).ok_or(ExecutorError::TaskNotFound(id))?;
        task.complete(result);
        // Wake any tasks waiting on this one
        self.wake_waiters_of(id);
        Ok(())
    }

    /// Cancel a task
    pub fn cancel_task(&mut self, id: TaskId) -> Result<(), ExecutorError> {
        let task = self.tasks.get_mut(&id).ok_or(ExecutorError::TaskNotFound(id))?;
        task.cancel();
        Ok(())
    }

    /// Make a task wait on another task
    pub fn wait_on(&mut self, waiter: TaskId, target: TaskId) -> Result<(), ExecutorError> {
        let task = self.tasks.get_mut(&waiter).ok_or(ExecutorError::TaskNotFound(waiter))?;
        task.wait_on(target);
        Ok(())
    }

    /// Wake all tasks waiting on a completed task
    fn wake_waiters_of(&mut self, completed: TaskId) {
        let waiters: Vec<TaskId> = self.tasks.values()
            .filter(|t| t.waiting_on == Some(completed))
            .map(|t| t.id)
            .collect();

        for waiter_id in waiters {
            if let Some(task) = self.tasks.get_mut(&waiter_id) {
                task.resume();
                self.ready_queue.push_back(waiter_id);
            }
        }
    }

    /// Check if any waiting tasks can be woken (their target is done)
    fn wake_completed_waiters(&mut self) {
        let done_ids: Vec<TaskId> = self.tasks.values()
            .filter(|t| t.is_done())
            .map(|t| t.id)
            .collect();

        for done_id in done_ids {
            self.wake_waiters_of(done_id);
        }
    }

    /// Get total ticks executed
    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Get all task results
    pub fn results(&self) -> Vec<(TaskId, Option<&TaskResult>)> {
        self.tasks.values()
            .map(|t| (t.id, t.result.as_ref()))
            .collect()
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
