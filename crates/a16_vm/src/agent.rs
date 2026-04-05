//! Agent Runtime
//!
//! Implements the Observe→Think→Act execution loop for A16 agents.
//! Agents coordinate models, tools, and memory for autonomous task execution.

use smol_str::SmolStr;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use a16_tensor::{MLP, Tensor, Activation};
use a16_vector::HNSWIndex;

use crate::tool_registry::{ToolRegistry, ToolResult};

/// Agent execution state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentState {
    /// Agent is idle, waiting for a task
    Idle,
    /// Agent is observing context (gathering memory/tool output)
    Observing,
    /// Agent is thinking (running model inference)
    Thinking,
    /// Agent is acting (dispatching tool calls)
    Acting,
    /// Agent is waiting on external input
    Waiting,
    /// Agent has completed its task
    Done,
    /// Agent encountered an error
    Error,
}

impl std::fmt::Display for AgentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentState::Idle => write!(f, "idle"),
            AgentState::Observing => write!(f, "observing"),
            AgentState::Thinking => write!(f, "thinking"),
            AgentState::Acting => write!(f, "acting"),
            AgentState::Waiting => write!(f, "waiting"),
            AgentState::Done => write!(f, "done"),
            AgentState::Error => write!(f, "error"),
        }
    }
}

/// Model handle wrapping a model configuration
#[derive(Debug, Clone)]
pub struct ModelHandle {
    /// Model name/identifier
    pub name: SmolStr,
    /// Temperature for generation (0.0 = deterministic, 1.0 = creative)
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Built-in MLP model (for local inference)
    pub mlp: Option<MLP>,
}

impl ModelHandle {
    /// Create a new model handle with default settings
    pub fn new(name: impl Into<SmolStr>) -> Self {
        Self {
            name: name.into(),
            temperature: 0.7,
            max_tokens: 1024,
            mlp: None,
        }
    }

    /// Create a model handle backed by a local MLP
    pub fn with_mlp(name: impl Into<SmolStr>, layer_sizes: &[usize]) -> Self {
        Self {
            name: name.into(),
            temperature: 0.0,
            max_tokens: 0,
            mlp: Some(MLP::new(layer_sizes, Activation::ReLU)),
        }
    }

    /// Run inference with the model
    pub fn generate(&self, input: &Tensor) -> Tensor {
        match &self.mlp {
            Some(mlp) => mlp.forward(input),
            None => {
                // Stub for external model — return input unchanged
                eprintln!("[ModelHandle] External model '{}' not connected, returning input", self.name);
                input.clone()
            }
        }
    }
}

/// Token budget tracker
#[derive(Debug, Clone)]
pub struct TokenBudget {
    /// Maximum total tokens allowed
    pub max_tokens: usize,
    /// Tokens consumed so far
    pub used_tokens: usize,
}

impl TokenBudget {
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens, used_tokens: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.max_tokens.saturating_sub(self.used_tokens)
    }

    pub fn consume(&mut self, tokens: usize) -> bool {
        if self.used_tokens + tokens <= self.max_tokens {
            self.used_tokens += tokens;
            true
        } else {
            false
        }
    }

    pub fn is_exhausted(&self) -> bool {
        self.used_tokens >= self.max_tokens
    }
}

/// Agent context — accumulated observations
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// Messages in the conversation
    pub messages: Vec<AgentMessage>,
    /// Tool results from this run
    pub tool_results: Vec<ToolResult>,
    /// Memory search results
    pub memory_hits: Vec<f32>,
}

impl AgentContext {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            tool_results: Vec::new(),
            memory_hits: Vec::new(),
        }
    }
}

impl Default for AgentContext {
    fn default() -> Self {
        Self::new()
    }
}

/// A message in the agent's conversation
#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub role: SmolStr,
    pub content: SmolStr,
}

/// The core Agent Runtime
///
/// Manages the Observe→Think→Act loop for autonomous task execution.
#[derive(Debug)]
pub struct AgentRuntime {
    /// Agent's unique name
    pub name: SmolStr,
    /// Current execution state
    pub state: AgentState,
    /// Model for inference
    pub model: ModelHandle,
    /// Tool registry available to this agent
    pub tools: Rc<RefCell<ToolRegistry>>,
    /// Vector memory index
    pub memory: Option<Rc<RefCell<HNSWIndex>>>,
    /// Token budget
    pub budget: TokenBudget,
    /// Accumulated context
    pub context: AgentContext,
    /// Step counter
    pub step_count: usize,
    /// Maximum steps before forced termination
    pub max_steps: usize,
    /// Agent-local variables (for task state)
    pub variables: HashMap<SmolStr, SmolStr>,
    /// Last result produced
    pub result: Option<SmolStr>,
}

impl AgentRuntime {
    /// Create a new agent runtime
    pub fn new(
        name: impl Into<SmolStr>,
        model: ModelHandle,
        tools: Rc<RefCell<ToolRegistry>>,
    ) -> Self {
        Self {
            name: name.into(),
            state: AgentState::Idle,
            model,
            tools,
            memory: None,
            budget: TokenBudget::new(10_000),
            context: AgentContext::new(),
            step_count: 0,
            max_steps: 100,
            variables: HashMap::new(),
            result: None,
        }
    }

    /// Attach a vector memory index to this agent
    pub fn with_memory(mut self, memory: Rc<RefCell<HNSWIndex>>) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Set the token budget
    pub fn with_budget(mut self, max_tokens: usize) -> Self {
        self.budget = TokenBudget::new(max_tokens);
        self
    }

    /// Set maximum steps
    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// Execute a single step of the Observe→Think→Act loop
    pub fn step(&mut self) -> AgentState {
        if self.state == AgentState::Done || self.state == AgentState::Error {
            return self.state;
        }

        if self.step_count >= self.max_steps {
            eprintln!("[Agent '{}'] Max steps ({}) reached", self.name, self.max_steps);
            self.state = AgentState::Done;
            return self.state;
        }

        if self.budget.is_exhausted() {
            eprintln!("[Agent '{}'] Token budget exhausted", self.name);
            self.state = AgentState::Done;
            return self.state;
        }

        self.step_count += 1;

        // === OBSERVE ===
        self.state = AgentState::Observing;
        self.observe();

        // === THINK ===
        self.state = AgentState::Thinking;
        let action = self.think();

        // === ACT ===
        self.state = AgentState::Acting;
        self.act(&action);

        // Check if we should continue or are done
        if action == "done" || action.is_empty() {
            self.state = AgentState::Done;
        } else {
            self.state = AgentState::Idle;
        }

        self.state
    }

    /// Run the agent until completion
    pub fn run(&mut self) -> SmolStr {
        self.state = AgentState::Idle;

        loop {
            let state = self.step();
            if state == AgentState::Done || state == AgentState::Error {
                break;
            }
        }

        self.result.clone().unwrap_or_else(|| SmolStr::new(""))
    }

    /// Observe phase: gather context from memory and environment
    fn observe(&mut self) {
        // Query memory if available
        if let Some(ref memory) = self.memory {
            let mem = memory.borrow();
            // Simple observation: check memory size
            let size = mem.len();
            if size > 0 {
                self.context.memory_hits.push(size as f32);
            }
        }

        // Consume tokens for observation
        self.budget.consume(10);
    }

    /// Think phase: process context and decide on action
    fn think(&mut self) -> String {
        // If we have an MLP model, run inference
        if self.model.mlp.is_some() {
            // Create a simple feature vector from context
            let num_messages = self.context.messages.len() as f32;
            let num_tool_results = self.context.tool_results.len() as f32;
            let steps = self.step_count as f32;
            let budget_remaining = self.budget.remaining() as f32 / self.budget.max_tokens as f32;

            let input = Tensor::from_data(
                vec![num_messages, num_tool_results, steps, budget_remaining],
                vec![1, 4],
            );

            let output = self.model.generate(&input);
            let data = output.data();

            // Interpret output: if any value > 0.5, continue; else done
            if data.iter().any(|&v| v > 0.5) {
                return "continue".to_string();
            }
        }

        // Consume tokens for thinking
        self.budget.consume(50);

        // Default: one-step execution then done
        "done".to_string()
    }

    /// Act phase: execute the decided action
    fn act(&mut self, action: &str) {
        match action {
            "done" => {
                self.result = Some(SmolStr::new(format!(
                    "Agent '{}' completed in {} steps",
                    self.name, self.step_count
                )));
            }
            _ => {
                // Try to call a tool with this action name
                let tools = self.tools.borrow();
                if let Some(result) = tools.call(action, &[]) {
                    self.context.tool_results.push(result);
                }
            }
        }

        // Consume tokens for acting
        self.budget.consume(20);
    }

    /// Get the agent's status summary
    pub fn status(&self) -> String {
        format!(
            "Agent '{}': state={}, steps={}/{}, tokens={}/{}",
            self.name, self.state, self.step_count, self.max_steps,
            self.budget.used_tokens, self.budget.max_tokens
        )
    }

    /// Add a message to the agent's context
    pub fn add_message(&mut self, role: impl Into<SmolStr>, content: impl Into<SmolStr>) {
        self.context.messages.push(AgentMessage {
            role: role.into(),
            content: content.into(),
        });
    }
}

impl Clone for AgentRuntime {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            state: self.state,
            model: self.model.clone(),
            tools: self.tools.clone(),
            memory: self.memory.clone(),
            budget: self.budget.clone(),
            context: self.context.clone(),
            step_count: self.step_count,
            max_steps: self.max_steps,
            variables: self.variables.clone(),
            result: self.result.clone(),
        }
    }
}
