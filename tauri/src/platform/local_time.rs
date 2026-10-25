//! The user's local day: where it starts, across daylight-saving changes,
//! by each system's time zone rules (`os/*/local_time.rs`).

/// One day, in milliseconds.
pub const DAY_MS: i64 = 86_400_000;

/// Start (UTC ms) of the local day `back` days before the day of `now`
/// (UTC ms); the UTC day when the system cannot tell.
pub fn day_start(now: i64, back: i64) -> i64 {
    crate::os::current::local_time::day_start(now, back)
}
