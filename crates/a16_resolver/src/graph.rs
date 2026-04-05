//! Module Dependency Graph
//!
//! Tracks module dependencies and provides topological ordering
//! for compilation.

use smol_str::SmolStr;
use indexmap::IndexMap;
use std::path::PathBuf;

/// Unique identifier for modules
pub type ModuleId = u32;

/// A node in the module dependency graph
#[derive(Debug, Clone)]
pub struct ModuleNode {
    /// Module ID
    pub id: ModuleId,
    /// Fully qualified module name (e.g., "myapp.utils.math")
    pub name: SmolStr,
    /// Path to the source file
    pub path: PathBuf,
    /// Source code content (loaded lazily)
    pub source: Option<String>,
    /// IDs of modules this module depends on
    pub dependencies: Vec<ModuleId>,
    /// IDs of modules that depend on this module
    pub dependents: Vec<ModuleId>,
    /// Whether this module has been compiled
    pub compiled: bool,
}

/// Module dependency graph for multi-file compilation
#[derive(Debug)]
pub struct ModuleGraph {
    /// All known modules
    modules: IndexMap<ModuleId, ModuleNode>,
    /// Map from module name to ID
    name_to_id: IndexMap<SmolStr, ModuleId>,
    /// Map from file path to module ID
    path_to_id: IndexMap<PathBuf, ModuleId>,
    /// Next available ID
    next_id: ModuleId,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            modules: IndexMap::new(),
            name_to_id: IndexMap::new(),
            path_to_id: IndexMap::new(),
            next_id: 0,
        }
    }

    /// Add a module to the graph
    pub fn add_module(&mut self, name: SmolStr, path: PathBuf) -> ModuleId {
        // Check if module already exists
        if let Some(&id) = self.name_to_id.get(&name) {
            return id;
        }

        let id = self.next_id;
        self.next_id += 1;

        let node = ModuleNode {
            id,
            name: name.clone(),
            path: path.clone(),
            source: None,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            compiled: false,
        };

        self.modules.insert(id, node);
        self.name_to_id.insert(name, id);
        self.path_to_id.insert(path, id);

        id
    }

    /// Add a dependency edge: `from` depends on `to`
    pub fn add_dependency(&mut self, from: ModuleId, to: ModuleId) {
        if let Some(node) = self.modules.get_mut(&from) {
            if !node.dependencies.contains(&to) {
                node.dependencies.push(to);
            }
        }
        if let Some(node) = self.modules.get_mut(&to) {
            if !node.dependents.contains(&from) {
                node.dependents.push(from);
            }
        }
    }

    /// Get a module by ID
    pub fn get_module(&self, id: ModuleId) -> Option<&ModuleNode> {
        self.modules.get(&id)
    }

    /// Get a module by name
    pub fn get_by_name(&self, name: &str) -> Option<&ModuleNode> {
        let id = self.name_to_id.get(name)?;
        self.modules.get(id)
    }

    /// Get a module by path
    pub fn get_by_path(&self, path: &PathBuf) -> Option<&ModuleNode> {
        let id = self.path_to_id.get(path)?;
        self.modules.get(id)
    }

    /// Number of modules in the graph
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Whether the graph is empty
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// Set source code for a module
    pub fn set_source(&mut self, id: ModuleId, source: String) {
        if let Some(node) = self.modules.get_mut(&id) {
            node.source = Some(source);
        }
    }

    /// Mark a module as compiled
    pub fn mark_compiled(&mut self, id: ModuleId) {
        if let Some(node) = self.modules.get_mut(&id) {
            node.compiled = true;
        }
    }

    /// Return a topological ordering of modules for compilation.
    /// Modules with no dependencies come first.
    pub fn topological_order(&self) -> Result<Vec<ModuleId>, CycleError> {
        let mut visited = IndexMap::new();
        let mut order = Vec::new();
        let mut stack = Vec::new();

        for &id in self.modules.keys() {
            if !visited.contains_key(&id) {
                self.topo_visit(id, &mut visited, &mut stack, &mut order)?;
            }
        }

        Ok(order)
    }

    fn topo_visit(
        &self,
        id: ModuleId,
        visited: &mut IndexMap<ModuleId, bool>,
        stack: &mut Vec<ModuleId>,
        order: &mut Vec<ModuleId>,
    ) -> Result<(), CycleError> {
        if let Some(&on_stack) = visited.get(&id) {
            if on_stack {
                // Cycle detected
                let cycle: Vec<SmolStr> = stack.iter()
                    .skip_while(|&&sid| sid != id)
                    .filter_map(|&sid| self.modules.get(&sid).map(|n| n.name.clone()))
                    .collect();
                return Err(CycleError { modules: cycle });
            }
            return Ok(()); // Already fully visited
        }

        visited.insert(id, true);
        stack.push(id);

        if let Some(node) = self.modules.get(&id) {
            for &dep_id in &node.dependencies {
                self.topo_visit(dep_id, visited, stack, order)?;
            }
        }

        stack.pop();
        visited.insert(id, false);
        order.push(id);

        Ok(())
    }

    /// Detect circular dependencies
    pub fn detect_cycles(&self) -> Option<CycleError> {
        match self.topological_order() {
            Ok(_) => None,
            Err(e) => Some(e),
        }
    }

    /// Get all module IDs
    pub fn module_ids(&self) -> Vec<ModuleId> {
        self.modules.keys().copied().collect()
    }
}

impl Default for ModuleGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Error for circular dependency detection
#[derive(Debug, Clone)]
pub struct CycleError {
    pub modules: Vec<SmolStr>,
}

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Circular dependency detected: {}", self.modules.iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join(" -> "))
    }
}

impl std::error::Error for CycleError {}
