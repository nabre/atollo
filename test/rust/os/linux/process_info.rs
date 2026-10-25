use super::*;

#[test]
fn start_time_from_proc_stat() {
    // Field 22 is the start time; the name may hold spaces and `)`.
    let stat = b"1234 (my (odd) prog) S 1 1234 1234 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 1 0 987654 1000 10 18446744073709551615";
    assert_eq!(stat_start_time(stat), Some(987_654));
    assert_eq!(
        stat_start_time(b"1 (init) S 1 1 1 0 -1 0 0 0 0 0 0 0 0 0 20 0 1 0 55"),
        Some(55)
    );
    assert_eq!(stat_start_time(b"1 (x) S 1 2"), None);
    assert_eq!(stat_start_time(b"no parenthesis"), None);
    assert_eq!(stat_start_time(b""), None);
}

#[test]
fn start_time_of_this_process() {
    let me = i32::try_from(std::process::id()).unwrap();
    let a = start_time(me).unwrap();
    assert_eq!(start_time(me), Some(a));
    assert_eq!(start_time(i32::MAX), None);
}
