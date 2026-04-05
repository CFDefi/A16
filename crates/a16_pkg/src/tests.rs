//! Package manager tests

use super::*;

#[test]
fn test_semver_parse() {
    let v = SemVer::parse("1.2.3").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 3);
}

#[test]
fn test_semver_display() {
    let v = SemVer::new(0, 1, 0);
    assert_eq!(v.to_string(), "0.1.0");
}

#[test]
fn test_semver_satisfies() {
    let req = SemVer::new(1, 0, 0);
    assert!(SemVer::new(1, 0, 0).satisfies(&req));
    assert!(SemVer::new(1, 1, 0).satisfies(&req));
    assert!(SemVer::new(1, 0, 5).satisfies(&req));
    assert!(!SemVer::new(2, 0, 0).satisfies(&req));
    assert!(!SemVer::new(0, 9, 0).satisfies(&req));
}

#[test]
fn test_manifest_new() {
    let m = Manifest::new("my_project");
    assert_eq!(m.package.name, "my_project");
    assert_eq!(m.package.version, "0.1.0");
    assert!(!m.has_dependencies());
}

#[test]
fn test_manifest_json_roundtrip() {
    let m = Manifest::new("test_pkg");
    let json = m.to_json().unwrap();
    let m2 = Manifest::from_json(&json).unwrap();
    assert_eq!(m.package.name, m2.package.name);
}

#[test]
fn test_lockfile_operations() {
    let mut lock = Lockfile::new();
    assert!(lock.is_empty());

    lock.lock("dep1".to_string(), LockedDependency {
        version: "1.0.0".to_string(),
        source: "registry".to_string(),
        integrity: Some("sha256-abc".to_string()),
        dependencies: vec![],
    });

    assert_eq!(lock.len(), 1);
    assert!(lock.is_locked("dep1"));
    assert!(!lock.is_locked("dep2"));
}

#[test]
fn test_lockfile_json_roundtrip() {
    let mut lock = Lockfile::new();
    lock.lock("mylib".to_string(), LockedDependency {
        version: "0.2.1".to_string(),
        source: "git".to_string(),
        integrity: None,
        dependencies: vec!["base".to_string()],
    });

    let json = lock.to_json().unwrap();
    let lock2 = Lockfile::from_json(&json).unwrap();
    assert_eq!(lock2.len(), 1);
    assert!(lock2.is_locked("mylib"));
}
