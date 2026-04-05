//! Tool Registry
//!
//! Manages tool registration, lookup, and sandboxed execution
//! for A16 agent tool dispatch.

use smol_str::SmolStr;
use std::collections::HashMap;
use std::time::Instant;

/// Permissions that a tool can request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Access network resources
    Network,
    /// Read files from the filesystem
    FileRead,
    /// Write files to the filesystem
    FileWrite,
    /// Execute external processes
    Execute,
    /// Access environment variables
    Environment,
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::Network => write!(f, "network"),
            Permission::FileRead => write!(f, "file_read"),
            Permission::FileWrite => write!(f, "file_write"),
            Permission::Execute => write!(f, "execute"),
            Permission::Environment => write!(f, "environment"),
        }
    }
}

/// Sandbox level for tool execution
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SandboxLevel {
    /// No restrictions
    None,
    /// Light sandboxing: log actions, enforce rate limits
    Light,
    /// Strict sandboxing: permission checks, resource limits
    Strict,
}

/// Rate limiter using token bucket algorithm
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Maximum calls per window
    max_calls: usize,
    /// Current window call count
    current_calls: usize,
    /// Window start time
    window_start: Option<Instant>,
    /// Window duration in seconds
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_calls_per_minute: usize) -> Self {
        Self {
            max_calls: max_calls_per_minute,
            current_calls: 0,
            window_start: None,
            window_secs: 60,
        }
    }

    pub fn unlimited() -> Self {
        Self {
            max_calls: usize::MAX,
            current_calls: 0,
            window_start: None,
            window_secs: 60,
        }
    }

    /// Check if a call is allowed and consume a token
    pub fn try_acquire(&mut self) -> bool {
        let now = Instant::now();

        // Reset window if expired
        if let Some(start) = self.window_start {
            if now.duration_since(start).as_secs() >= self.window_secs {
                self.current_calls = 0;
                self.window_start = Some(now);
            }
        } else {
            self.window_start = Some(now);
        }

        if self.current_calls < self.max_calls {
            self.current_calls += 1;
            true
        } else {
            false
        }
    }
}

/// Result from a tool execution
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Name of the tool that produced this result
    pub tool_name: SmolStr,
    /// Whether the tool succeeded
    pub success: bool,
    /// Output data
    pub output: SmolStr,
    /// Error message if failed
    pub error: Option<SmolStr>,
}

/// A registered tool definition
#[derive(Debug, Clone)]
pub struct ToolDef {
    /// Tool name
    pub name: SmolStr,
    /// Human-readable description
    pub description: SmolStr,
    /// Required permissions
    pub permissions: Vec<Permission>,
    /// Sandbox level
    pub sandbox: SandboxLevel,
    /// Rate limiter
    pub rate_limiter: RateLimiter,
    /// Built-in handler function
    pub handler: ToolHandler,
}

/// Tool handler — either a built-in function or a name to resolve at runtime
#[derive(Debug, Clone)]
pub enum ToolHandler {
    /// Built-in tool with a static function
    BuiltIn(fn(&[SmolStr]) -> ToolResult),
    /// User-defined tool (resolved by name in the VM)
    UserDefined(SmolStr),
}

/// Tool Registry - manages all available tools
#[derive(Debug)]
pub struct ToolRegistry {
    tools: HashMap<SmolStr, ToolDef>,
    /// Granted permissions for this execution context
    granted_permissions: Vec<Permission>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            granted_permissions: vec![
                Permission::Network,
                Permission::FileRead,
                Permission::FileWrite,
                Permission::Execute,
                Permission::Environment,
            ],
        };

        // Register built-in tools
        registry.register_builtins();
        registry
    }

    /// Register a tool
    pub fn register(&mut self, tool: ToolDef) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Look up a tool by name
    pub fn get(&self, name: &str) -> Option<&ToolDef> {
        self.tools.get(name)
    }

    /// List all registered tool names
    pub fn list(&self) -> Vec<SmolStr> {
        self.tools.keys().cloned().collect()
    }

    /// Call a tool by name with arguments
    pub fn call(&self, name: &str, args: &[SmolStr]) -> Option<ToolResult> {
        let tool = self.tools.get(name)?;

        // Check permissions
        for perm in &tool.permissions {
            if !self.granted_permissions.contains(perm) {
                return Some(ToolResult {
                    tool_name: tool.name.clone(),
                    success: false,
                    output: SmolStr::new(""),
                    error: Some(SmolStr::new(format!(
                        "Permission '{}' not granted for tool '{}'",
                        perm, tool.name
                    ))),
                });
            }
        }

        // Execute handler
        match &tool.handler {
            ToolHandler::BuiltIn(f) => Some(f(args)),
            ToolHandler::UserDefined(fname) => {
                Some(ToolResult {
                    tool_name: tool.name.clone(),
                    success: true,
                    output: SmolStr::new(format!("[user_tool:{}]", fname)),
                    error: None,
                })
            }
        }
    }

    /// Number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Whether registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Register built-in tools
    fn register_builtins(&mut self) {
        // Calculator tool
        self.register(ToolDef {
            name: SmolStr::new("calculator"),
            description: SmolStr::new("Evaluate mathematical expressions"),
            permissions: vec![],
            sandbox: SandboxLevel::Strict,
            rate_limiter: RateLimiter::unlimited(),
            handler: ToolHandler::BuiltIn(builtin_calculator),
        });

        // Echo tool (for testing)
        self.register(ToolDef {
            name: SmolStr::new("echo"),
            description: SmolStr::new("Echo back the input"),
            permissions: vec![],
            sandbox: SandboxLevel::None,
            rate_limiter: RateLimiter::unlimited(),
            handler: ToolHandler::BuiltIn(builtin_echo),
        });

        // Clock tool
        self.register(ToolDef {
            name: SmolStr::new("clock"),
            description: SmolStr::new("Get current timestamp"),
            permissions: vec![],
            sandbox: SandboxLevel::None,
            rate_limiter: RateLimiter::unlimited(),
            handler: ToolHandler::BuiltIn(builtin_clock),
        });
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ToolRegistry {
    fn clone(&self) -> Self {
        Self {
            tools: self.tools.clone(),
            granted_permissions: self.granted_permissions.clone(),
        }
    }
}

// === Built-in Tool Implementations ===

fn builtin_calculator(args: &[SmolStr]) -> ToolResult {
    let expr = args.first().map(|s| s.as_str()).unwrap_or("0");

    // Simple expression evaluator for basic arithmetic
    let result = match expr.trim() {
        s if s.contains('+') => {
            let parts: Vec<&str> = s.splitn(2, '+').collect();
            let a: f64 = parts[0].trim().parse().unwrap_or(0.0);
            let b: f64 = parts[1].trim().parse().unwrap_or(0.0);
            format!("{}", a + b)
        }
        s if s.contains('-') && !s.starts_with('-') => {
            let parts: Vec<&str> = s.splitn(2, '-').collect();
            let a: f64 = parts[0].trim().parse().unwrap_or(0.0);
            let b: f64 = parts[1].trim().parse().unwrap_or(0.0);
            format!("{}", a - b)
        }
        s if s.contains('*') => {
            let parts: Vec<&str> = s.splitn(2, '*').collect();
            let a: f64 = parts[0].trim().parse().unwrap_or(0.0);
            let b: f64 = parts[1].trim().parse().unwrap_or(0.0);
            format!("{}", a * b)
        }
        s if s.contains('/') => {
            let parts: Vec<&str> = s.splitn(2, '/').collect();
            let a: f64 = parts[0].trim().parse().unwrap_or(0.0);
            let b: f64 = parts[1].trim().parse().unwrap_or(1.0);
            if b == 0.0 {
                return ToolResult {
                    tool_name: SmolStr::new("calculator"),
                    success: false,
                    output: SmolStr::new(""),
                    error: Some(SmolStr::new("Division by zero")),
                };
            }
            format!("{}", a / b)
        }
        s => {
            // Try to parse as a number
            s.parse::<f64>()
                .map(|n| format!("{}", n))
                .unwrap_or_else(|_| "0".to_string())
        }
    };

    ToolResult {
        tool_name: SmolStr::new("calculator"),
        success: true,
        output: SmolStr::new(result),
        error: None,
    }
}

fn builtin_echo(args: &[SmolStr]) -> ToolResult {
    let output = args.iter()
        .map(|s| s.as_str())
        .collect::<Vec<&str>>()
        .join(" ");

    ToolResult {
        tool_name: SmolStr::new("echo"),
        success: true,
        output: SmolStr::new(output),
        error: None,
    }
}

fn builtin_clock(_args: &[SmolStr]) -> ToolResult {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    ToolResult {
        tool_name: SmolStr::new("clock"),
        success: true,
        output: SmolStr::new(format!("{}", timestamp)),
        error: None,
    }
}
