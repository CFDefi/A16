//! A16 Virtual Machine
//!
//! Stack-based bytecode interpreter for A16 programs.

mod value;
mod vm;
mod stdlib;
pub mod agent;
pub mod tool_registry;

pub use value::Value;
pub use vm::VM;
pub use stdlib::register_stdlib;
pub use agent::{AgentRuntime, AgentState, ModelHandle};
pub use tool_registry::{ToolRegistry, ToolDef, ToolResult, Permission, SandboxLevel};

// Re-export async runtime types for convenience
pub use a16_runtime::{Channel, ChannelMessage, ChannelError};
pub use a16_runtime::{Executor, ExecutorError};
pub use a16_runtime::{Task, TaskId, TaskState, TaskResult, TaskPriority};

// Re-export FFI types for convenience
pub use a16_ffi::{FfiRegistry, FfiType, FfiValue, FfiFunc, FfiError, FfiCallback};

#[cfg(test)]
mod tests;
