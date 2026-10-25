//! The local day (`platform::local_time`) by the C library's time zone
//! rules.

use crate::platform::local_time::DAY_MS;

/// Start (UTC ms) of the local day `back` days before the day of `now`,
/// right across daylight-saving changes (the C library's time zone rules).
pub fn day_start(now: i64, back: i64) -> i64 {
    let utc_days = now.div_euclid(DAY_MS) * DAY_MS - back * DAY_MS;
    #[allow(clippy::unnecessary_cast)]
    let secs = now.div_euclid(1000) as libc::time_t;
    // SAFETY: localtime_r and mktime read and write only the local `tm`.
    unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&secs, &mut tm).is_null() {
            return utc_days;
        }
        tm.tm_hour = 0;
        tm.tm_min = 0;
        tm.tm_sec = 0;
        // mktime normalises a day of the month below 1 into earlier months.
        tm.tm_mday -= i32::try_from(back).unwrap_or(0);
        tm.tm_isdst = -1;
        match libc::mktime(&mut tm) {
            -1 => utc_days,
            #[allow(clippy::useless_conversion)]
            start => i64::from(start) * 1000,
        }
    }
}
