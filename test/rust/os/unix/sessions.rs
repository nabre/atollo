use super::*;
use std::time::Duration;

/// D-105: `SIGTERM` to all, `SIGKILL` only to those still running after
/// the wait; nothing waits when all have ended.
#[test]
fn gentle_stop_waits_then_kills() {
    use std::cell::RefCell;
    let sent = RefCell::new(Vec::new());
    let stubborn = 7;
    stop_gently(
        &[5, stubborn],
        Duration::from_millis(200),
        Duration::from_millis(10),
        |pid, s| sent.borrow_mut().push((pid, s)),
        // 5 ends on SIGTERM: running when first looked at, then gone.
        |pid| match pid {
            7 => Some(70),
            _ if sent.borrow().is_empty() => Some(50),
            _ => None,
        },
    );
    assert_eq!(
        sent.into_inner(),
        [
            (5, Signal::Term),
            (stubborn, Signal::Term),
            (stubborn, Signal::Kill)
        ]
    );
    let sent = RefCell::new(Vec::new());
    let start = std::time::Instant::now();
    stop_gently(
        &[5],
        Duration::from_secs(5),
        Duration::from_millis(10),
        |pid, s| sent.borrow_mut().push((pid, s)),
        |_| sent.borrow().is_empty().then_some(50),
    );
    assert!(start.elapsed() < Duration::from_secs(1));
    assert_eq!(sent.into_inner(), [(5, Signal::Term)]);
}

/// A pid that ends during the wait and is taken by another program (a
/// new start time) gets no `SIGKILL`: that program is not the kit's.
#[test]
fn gentle_stop_spares_a_reused_pid() {
    use std::cell::{Cell, RefCell};
    let sent = RefCell::new(Vec::new());
    let calls = Cell::new(0);
    stop_gently(
        &[9],
        Duration::from_millis(100),
        Duration::from_millis(10),
        |pid, s| sent.borrow_mut().push((pid, s)),
        |_| {
            calls.set(calls.get() + 1);
            // The kit's program, then from the second look another one.
            Some(if calls.get() == 1 { 1000 } else { 2000 })
        },
    );
    assert_eq!(sent.into_inner(), [(9, Signal::Term)]);
}
