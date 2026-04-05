//! Module resolver tests

use super::*;
use std::path::PathBuf;
use smol_str::SmolStr;

#[test]
fn test_module_graph_add_module() {
    let mut graph = ModuleGraph::new();
    let id = graph.add_module(SmolStr::new("main"), PathBuf::from("main.a16"));
    assert_eq!(graph.len(), 1);
    let node = graph.get_module(id).unwrap();
    assert_eq!(node.name.as_str(), "main");
}

#[test]
fn test_module_graph_idempotent() {
    let mut graph = ModuleGraph::new();
    let id1 = graph.add_module(SmolStr::new("utils"), PathBuf::from("utils.a16"));
    let id2 = graph.add_module(SmolStr::new("utils"), PathBuf::from("utils.a16"));
    assert_eq!(id1, id2);
    assert_eq!(graph.len(), 1);
}

#[test]
fn test_module_graph_dependencies() {
    let mut graph = ModuleGraph::new();
    let main_id = graph.add_module(SmolStr::new("main"), PathBuf::from("main.a16"));
    let utils_id = graph.add_module(SmolStr::new("utils"), PathBuf::from("utils.a16"));
    graph.add_dependency(main_id, utils_id);

    let main = graph.get_module(main_id).unwrap();
    assert!(main.dependencies.contains(&utils_id));
    let utils = graph.get_module(utils_id).unwrap();
    assert!(utils.dependents.contains(&main_id));
}

#[test]
fn test_topological_order() {
    let mut graph = ModuleGraph::new();
    let a = graph.add_module(SmolStr::new("a"), PathBuf::from("a.a16"));
    let b = graph.add_module(SmolStr::new("b"), PathBuf::from("b.a16"));
    let c = graph.add_module(SmolStr::new("c"), PathBuf::from("c.a16"));
    
    // c depends on b, b depends on a
    graph.add_dependency(c, b);
    graph.add_dependency(b, a);
    
    let order = graph.topological_order().unwrap();
    let a_pos = order.iter().position(|&id| id == a).unwrap();
    let b_pos = order.iter().position(|&id| id == b).unwrap();
    let c_pos = order.iter().position(|&id| id == c).unwrap();
    
    // a must come before b, b before c
    assert!(a_pos < b_pos);
    assert!(b_pos < c_pos);
}

#[test]
fn test_cycle_detection() {
    let mut graph = ModuleGraph::new();
    let a = graph.add_module(SmolStr::new("a"), PathBuf::from("a.a16"));
    let b = graph.add_module(SmolStr::new("b"), PathBuf::from("b.a16"));
    
    // a depends on b, b depends on a
    graph.add_dependency(a, b);
    graph.add_dependency(b, a);
    
    let cycle = graph.detect_cycles();
    assert!(cycle.is_some());
}

#[test]
fn test_resolver_builtin_std() {
    let resolver = Resolver::new(PathBuf::from("."));
    let result = resolver.resolve("std.io", None).unwrap();
    assert!(result.is_stdlib);
    assert_eq!(result.name.as_str(), "std.io");
}

#[test]
fn test_module_graph_lookup_by_name() {
    let mut graph = ModuleGraph::new();
    graph.add_module(SmolStr::new("mymod"), PathBuf::from("mymod.a16"));
    
    let node = graph.get_by_name("mymod");
    assert!(node.is_some());
    assert_eq!(node.unwrap().name.as_str(), "mymod");
    
    assert!(graph.get_by_name("nonexistent").is_none());
}

#[test]
fn test_set_source_and_compile() {
    let mut graph = ModuleGraph::new();
    let id = graph.add_module(SmolStr::new("main"), PathBuf::from("main.a16"));
    
    graph.set_source(id, "fn main(): pass".to_string());
    assert!(graph.get_module(id).unwrap().source.is_some());
    
    graph.mark_compiled(id);
    assert!(graph.get_module(id).unwrap().compiled);
}
