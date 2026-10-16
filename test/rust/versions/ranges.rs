use super::*;

fn r(text: &str) -> Range {
    Range::parse(text, Bare::Exact).unwrap_or_else(|| panic!("{text}"))
}

#[test]
fn versions_in_text() {
    assert_eq!(Version::parse("v22.11.0"), Some(Version(22, 11, 0)));
    assert_eq!(Version::parse("go1.25.3"), Some(Version(1, 25, 3)));
    assert_eq!(
        Version::parse("8.4.13 (Composer 2.8.12)"),
        Some(Version(8, 4, 13))
    );
    assert_eq!(Version::parse("3.15.0a1"), Some(Version(3, 15, 0)));
    assert_eq!(Version::parse("21"), Some(Version(21, 0, 0)));
    assert_eq!(Version::parse("none"), None);
}

#[test]
fn npm_and_composer() {
    let caret = r("^20.11.0");
    assert!(caret.admits_text("20.11.0") && caret.admits_text("20.19.5"));
    assert!(!caret.admits_text("20.10.9") && !caret.admits_text("21.0.0"));
    assert!(r("^0.2.3").admits_text("0.2.9") && !r("^0.2.3").admits_text("0.3.0"));
    assert!(r("~8.2").admits_text("8.2.30") && !r("~8.2").admits_text("8.3.0"));
    assert!(r(">=18 <23").admits_text("22.21.1") && !r(">=18 <23").admits_text("23.0.0"));
    assert!(r(">= 18.12").admits_text("18.12.0"));
    assert!(r("8.1.x").admits_text("8.1.9") && !r("8.1.x").admits_text("8.2.0"));
    assert!(r("1.2 - 1.4").admits_text("1.4.7") && !r("1.2 - 1.4").admits_text("1.5.0"));
    let or = r("^8.2 || ^9.0");
    assert!(or.admits_text("8.4.13") && or.admits_text("9.1.0") && !or.admits_text("8.1.0"));
    assert!(r("^7.4|^8.0").admits_text("8.4.1"), "Composer's single bar");
    assert!(r("*").admits_text("1.0.0"));
    assert!(r(">8").admits_text("9.0.0") && !r(">8").admits_text("8.9.0"));
}

#[test]
fn pep_440() {
    let py = r(">=3.10,<3.13");
    assert!(py.admits_line("3.12") && !py.admits_line("3.13") && !py.admits_line("3.9"));
    assert!(r("~=3.11").admits_text("3.14.0") && !r("~=3.11").admits_text("4.0.0"));
    assert!(r("~=3.11.2").admits_text("3.11.9") && !r("~=3.11.2").admits_text("3.12.0"));
    assert!(r("==3.12.*").admits_line("3.12") && !r("==3.12.*").admits_line("3.13"));
    assert!(!r(">=3.9, !=3.11.4").admits_text("3.11.4"));
}

#[test]
fn minimums_and_lines() {
    let go = Range::parse("1.21", Bare::Minimum).unwrap();
    assert!(go.admits_line("1.25") && !go.admits_line("1.20"));
    assert!(r("^20.11.0").admits_line("20.19") && !r("^20.11.0").admits_line("20.10"));
    assert!(r("8.2").admits_line("8.2") && !r("8.2").admits_line("8.3"));
    assert!(Range::parse("", Bare::Exact).is_none());
    assert!(Range::parse("latest", Bare::Exact).is_none());
}
