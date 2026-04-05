//! Async runtime tests

use super::*;
use smol_str::SmolStr;

// === Executor Tests ===

#[test]
fn test_executor_spawn() {
    let mut exec = Executor::new();
    let id = exec.spawn(SmolStr::new("task1"), 0);
    assert_eq!(exec.total_count(), 1);
    assert_eq!(exec.active_count(), 1);
    assert!(exec.get_task(id).is_some());
}

#[test]
fn test_executor_tick() {
    let mut exec = Executor::new();
    exec.spawn(SmolStr::new("task1"), 0);
    let result = exec.tick().unwrap();
    assert!(result.is_some());
    assert_eq!(exec.done_count(), 1);
}

#[test]
fn test_executor_run_all() {
    let mut exec = Executor::new();
    exec.spawn(SmolStr::new("t1"), 0);
    exec.spawn(SmolStr::new("t2"), 1);
    exec.spawn(SmolStr::new("t3"), 2);
    exec.run_all().unwrap();
    assert_eq!(exec.done_count(), 3);
    assert_eq!(exec.active_count(), 0);
}

#[test]
fn test_executor_cancel() {
    let mut exec = Executor::new();
    let id = exec.spawn(SmolStr::new("cancel_me"), 0);
    exec.cancel_task(id).unwrap();
    let task = exec.get_task(id).unwrap();
    assert!(task.is_done());
    assert!(matches!(task.result, Some(TaskResult::Cancelled)));
}

#[test]
fn test_executor_max_ticks() {
    let mut exec = Executor::new().with_max_ticks(2);
    exec.spawn(SmolStr::new("t1"), 0);
    exec.spawn(SmolStr::new("t2"), 0);
    exec.spawn(SmolStr::new("t3"), 0);
    let result = exec.run_all();
    assert!(result.is_err());
}

#[test]
fn test_executor_spawn_with_priority() {
    let mut exec = Executor::new();
    let normal = exec.spawn(SmolStr::new("normal"), 0);
    let high = exec.spawn_with_priority(SmolStr::new("high"), 1, TaskPriority::High);

    // High priority should run first
    let first = exec.tick().unwrap().unwrap();
    assert_eq!(first, high);

    let second = exec.tick().unwrap().unwrap();
    assert_eq!(second, normal);
}

#[test]
fn test_executor_poll_any_ready() {
    let mut exec = Executor::new();
    assert!(exec.poll_any_ready().is_none());

    let id = exec.spawn(SmolStr::new("task"), 0);
    assert_eq!(exec.poll_any_ready(), Some(id));
}

#[test]
fn test_executor_join_all() {
    let mut exec = Executor::new();
    let id1 = exec.spawn(SmolStr::new("t1"), 0);
    let id2 = exec.spawn(SmolStr::new("t2"), 1);
    let id3 = exec.spawn(SmolStr::new("t3"), 2);

    let results = exec.join_all(&[id1, id2, id3]).unwrap();
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_some()));
}

// === Task Tests ===

#[test]
fn test_task_lifecycle() {
    let mut task = Task::new(1, SmolStr::new("test"), 0);
    assert_eq!(task.state, TaskState::Ready);
    assert!(!task.is_done());

    task.state = TaskState::Running;
    assert!(!task.is_done());

    task.complete(TaskResult::Value(SmolStr::new("42")));
    assert!(task.is_done());
    assert_eq!(task.state, TaskState::Done);
}

#[test]
fn test_task_wait_resume() {
    let mut task = Task::new(1, SmolStr::new("waiter"), 0);
    task.wait_on(2);
    assert_eq!(task.state, TaskState::Waiting);
    assert_eq!(task.waiting_on, Some(2));

    task.resume();
    assert_eq!(task.state, TaskState::Ready);
    assert_eq!(task.waiting_on, None);
}

#[test]
fn test_task_priority() {
    let task = Task::new(1, SmolStr::new("normal"), 0);
    assert_eq!(task.priority, TaskPriority::Normal);

    let high_task = Task::new(2, SmolStr::new("high"), 0).with_priority(TaskPriority::High);
    assert_eq!(high_task.priority, TaskPriority::High);
}

#[test]
fn test_task_result_value_id() {
    let mut task = Task::new(1, SmolStr::new("vm_task"), 0);
    task.complete(TaskResult::ValueId(42));
    assert!(task.is_done());
    if let Some(TaskResult::ValueId(id)) = &task.result {
        assert_eq!(*id, 42);
    } else {
        panic!("Expected ValueId result");
    }
}

// === Channel Tests ===

#[test]
fn test_channel_send_recv() {
    let mut ch = Channel::new(SmolStr::new("ch1"), 10);
    ch.send_text(SmolStr::new("hello")).unwrap();
    ch.send_text(SmolStr::new("world")).unwrap();
    assert_eq!(ch.len(), 2);

    let msg = ch.recv().unwrap();
    assert_eq!(msg.as_text().unwrap(), "hello");
    assert_eq!(ch.len(), 1);
}

#[test]
fn test_channel_capacity() {
    let mut ch = Channel::new(SmolStr::new("small"), 2);
    ch.send_text(SmolStr::new("a")).unwrap();
    ch.send_text(SmolStr::new("b")).unwrap();
    assert!(ch.is_full());
    assert!(ch.send_text(SmolStr::new("c")).is_err());
}

#[test]
fn test_channel_close() {
    let mut ch = Channel::new(SmolStr::new("ch"), 5);
    ch.send_text(SmolStr::new("before_close")).unwrap();
    ch.close();
    assert!(ch.is_closed());
    assert!(ch.send_text(SmolStr::new("after_close")).is_err());
    // Can still recv buffered messages
    assert!(ch.recv().is_ok());
    // But empty+closed returns Closed error
    assert!(ch.recv().is_err());
}

#[test]
fn test_channel_try_recv() {
    let mut ch = Channel::new(SmolStr::new("ch"), 5);
    assert!(ch.try_recv().is_none());
    ch.send_text(SmolStr::new("msg")).unwrap();
    let msg = ch.try_recv().unwrap();
    assert_eq!(msg.as_text().unwrap(), "msg");
}

#[test]
fn test_channel_stats() {
    let mut ch = Channel::new(SmolStr::new("stats"), 10);
    ch.send_text(SmolStr::new("a")).unwrap();
    ch.send_text(SmolStr::new("b")).unwrap();
    ch.recv().unwrap();
    assert_eq!(ch.total_sent(), 2);
    assert_eq!(ch.total_received(), 1);
}

#[test]
fn test_channel_value_id_messages() {
    let mut ch = Channel::new(SmolStr::new("vm_ch"), 10);
    ch.send(ChannelMessage::ValueId(42)).unwrap();
    ch.send(ChannelMessage::ValueId(99)).unwrap();

    let msg1 = ch.recv().unwrap();
    assert_eq!(msg1.as_value_id().unwrap(), 42);

    let msg2 = ch.recv().unwrap();
    assert_eq!(msg2.as_value_id().unwrap(), 99);
}

#[test]
fn test_channel_mixed_messages() {
    let mut ch = Channel::new(SmolStr::new("mixed"), 10);
    ch.send(ChannelMessage::Text(SmolStr::new("hello"))).unwrap();
    ch.send(ChannelMessage::ValueId(100)).unwrap();

    let msg1 = ch.recv().unwrap();
    assert!(msg1.as_text().is_some());
    assert!(msg1.as_value_id().is_none());

    let msg2 = ch.recv().unwrap();
    assert!(msg2.as_text().is_none());
    assert!(msg2.as_value_id().is_some());
}
