//! A16 Async Runtime
//!
//! Provides cooperative multitasking for A16 programs.
//! - `Executor`: single-threaded task scheduler
//! - `TaskQueue`: ready/waiting task management
//! - `Channel`: bounded MPSC message passing
//! - `Future`: boxed async computation state

mod executor;
mod channel;
mod task;

pub use executor::{Executor, ExecutorError};
pub use channel::{Channel, ChannelError, ChannelMessage};
pub use task::{Task, TaskId, TaskState, TaskResult, TaskPriority};

#[cfg(test)]
mod tests;
