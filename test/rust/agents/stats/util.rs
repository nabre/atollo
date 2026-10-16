use super::*;

#[test]
fn iso_times() {
    assert_eq!(parse_iso("1970-01-01T00:00:00Z"), Some(0));
    assert_eq!(parse_iso("1970-01-02"), Some(DAY_MS));
    assert_eq!(parse_iso("2026-10-14T08:12:33.5Z"), Some(1_791_965_553_500));
    assert_eq!(
        parse_iso("2026-10-14T10:12:33.500+02:00"),
        parse_iso("2026-10-14T08:12:33.500Z")
    );
    assert_eq!(
        parse_iso("2026-10-14T03:12:33.123456-0500"),
        Some(1_791_965_553_123)
    );
    for bad in [
        "",
        "yesterday",
        "2026-13-01",
        "2026-10-14T08",
        "2026-10-14T08:12+2",
    ] {
        assert_eq!(parse_iso(bad), None, "{bad}");
    }
    assert_eq!(
        to_time(Some(&Value::from(1_791_965_553))),
        Some(1_791_965_553_000)
    );
    assert_eq!(
        to_time(Some(&Value::from(1_791_965_553_000_i64))),
        Some(1_791_965_553_000)
    );
    assert_eq!(to_time(Some(&Value::Null)), None);
}

#[test]
fn retry_after_values() {
    assert_eq!(retry_after(None, 0), 0);
    assert_eq!(retry_after(Some("120"), 0), 120_000);
    assert_eq!(retry_after(Some("999999"), 0), 6 * 3600 * 1000);
    let at = parse_iso("2026-10-21T07:28:00Z").unwrap();
    assert_eq!(
        retry_after(Some("Wed, 21 Oct 2026 07:28:00 GMT"), at - 5000),
        5000
    );
    assert_eq!(retry_after(Some("soon"), 0), 0);
}

#[test]
fn local_days_are_a_day_apart() {
    let p = Period::now();
    for w in p.days.windows(2) {
        // 23, 24 or 25 hours across daylight-saving changes
        let hours = (w[1] - w[0]) / 3_600_000;
        assert!((23..=25).contains(&hours), "{hours}");
    }
    assert!(p.today() <= now_ms() && now_ms() - p.today() < DAY_MS + 3_600_000);
}

#[test]
fn summary_of_the_period() {
    let p = Period {
        days: std::array::from_fn(|i| i as i64 * DAY_MS),
    };
    let rec = |time: i64, model: &str, n: u64| Record {
        time: Some(time),
        model: Some(model.into()),
        tokens: Tokens::new(n, 1, 2),
    };
    let records = [
        rec(-1, "old", 100),
        rec(10, "a", 10),
        rec(6 * DAY_MS + 5, "a", 20),
        rec(9 * DAY_MS, "b", 30),
        rec(DAY_MS, "<synthetic>", 0),
    ];
    let u = summarize(&records, &[3 * DAY_MS, 6 * DAY_MS, -5], true, &p);
    assert_eq!(u.week, Tokens::new(60, 4, 8));
    assert_eq!(u.today, Tokens::new(50, 2, 4), "a clock ahead counts today");
    assert_eq!(u.days[0].tokens.total, 13);
    assert_eq!((u.sessions_week, u.sessions_today), (2, 1));
    assert_eq!(u.days[3].sessions, 1);
    assert_eq!(u.last, Some(9 * DAY_MS));
    assert_eq!(u.models.get("a"), Some(&36));
    assert!(!u.models.contains_key("<synthetic>"));
    assert_eq!(u.since, 0);
}

#[test]
fn levels() {
    assert_eq!(level(None), "none");
    assert_eq!(level(Some(69.9)), "ok");
    assert_eq!(level(Some(70.0)), "warning");
    assert_eq!(level(Some(90.0)), "critical");
    assert_eq!(clamp_percent(123.456), Some(100.0));
    assert_eq!(clamp_percent(12.345), Some(12.3));
    assert_eq!(clamp_percent(f64::NAN), None);
}
