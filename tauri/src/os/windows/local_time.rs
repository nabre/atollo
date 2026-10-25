//! The local day (`platform::local_time`) by Windows' time zone rules.

use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows::Win32::System::Time::{
    FileTimeToSystemTime, SystemTimeToFileTime, SystemTimeToTzSpecificLocalTime,
    TzSpecificLocalTimeToSystemTime,
};

use crate::platform::local_time::DAY_MS;

/// Start (UTC ms) of the local day `back` days before the day of `now`,
/// right across daylight-saving changes (Windows' time zone rules).
pub fn day_start(now: i64, back: i64) -> i64 {
    const EPOCH_DIFF_MS: i64 = 11_644_473_600_000;
    let to_ft = |ms: i64| {
        let ticks = ((ms + EPOCH_DIFF_MS) * 10_000) as u64;
        FILETIME {
            dwLowDateTime: ticks as u32,
            dwHighDateTime: (ticks >> 32) as u32,
        }
    };
    let of_ft = |ft: FILETIME| {
        let ticks = (u64::from(ft.dwHighDateTime) << 32) | u64::from(ft.dwLowDateTime);
        (ticks / 10_000) as i64 - EPOCH_DIFF_MS
    };
    let utc_days = now.div_euclid(DAY_MS) * DAY_MS - back * DAY_MS;
    // SAFETY: every call reads and writes only the local structures passed.
    unsafe {
        let (mut utc, mut local) = (SYSTEMTIME::default(), SYSTEMTIME::default());
        if FileTimeToSystemTime(&to_ft(now), &mut utc).is_err()
            || SystemTimeToTzSpecificLocalTime(None, &utc, &mut local).is_err()
        {
            return utc_days;
        }
        local.wHour = 0;
        local.wMinute = 0;
        local.wSecond = 0;
        local.wMilliseconds = 0;
        let mut naive = FILETIME::default();
        if SystemTimeToFileTime(&local, &mut naive).is_err() {
            return utc_days;
        }
        let mut day = SYSTEMTIME::default();
        let mut start = SYSTEMTIME::default();
        let mut out = FILETIME::default();
        if FileTimeToSystemTime(&to_ft(of_ft(naive) - back * DAY_MS), &mut day).is_err()
            || TzSpecificLocalTimeToSystemTime(None, &day, &mut start).is_err()
            || SystemTimeToFileTime(&start, &mut out).is_err()
        {
            return utc_days;
        }
        of_ft(out)
    }
}
