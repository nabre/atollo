use super::*;

#[test]
fn editor_stays_in_its_project() {
    use crate::kit::Os;
    let u = |s: &str| s.parse::<Url>().unwrap();
    let in_project = |url: &Url, dir: &Path| in_project_on(Os::Win32, url, dir);
    let dir = Path::new(r"E:\Atollo (1)\projects\app & co");
    let page = crate::editor::url_on(Os::Win32, 7003, "tok", Some(dir));
    assert!(in_project(&u(&page), dir));
    // after the server's redirect: the token goes, the folder stays
    assert!(in_project(
        &u("http://127.0.0.1:7003/?folder=/e:/Dev%20Kit%20(1)/projects/app%20%26%20co/"),
        dir
    ));
    // a share keeps its three slashes, as before macOS and Linux
    let share = Path::new(r"\\nas\Kit\projects\app");
    let page = crate::editor::url_on(Os::Win32, 7003, "tok", Some(share));
    assert!(page.contains("folder=///nas/Kit/"), "{page}");
    assert!(in_project(&u(&page), share));
    assert!(in_project(
        &u("http://127.0.0.1:7003/?folder=///NAS/kit/projects/app"),
        share
    ));
    assert!(!in_project(
        &u("http://127.0.0.1:7003/?folder=//nas/Kit/projects/app"),
        share
    ));
    let page = crate::editor::url_on(Os::Win32, 7003, "tok", Some(dir));
    for out in [
        "http://127.0.0.1:7003/",
        "http://127.0.0.1:7003/?ew=true",
        "http://127.0.0.1:7003/?folder=/E:/Dev%20Kit%20(1)/projects",
        "http://127.0.0.1:7003/?folder=/E:/Dev%20Kit%20(1)/projects/app%20%26%20co/sub",
        "http://127.0.0.1:7003/?folder=/C:/Users",
        "http://127.0.0.1:7003/?workspace=/E:/x.code-workspace",
    ] {
        assert!(!in_project(&u(out), dir), "{out}");
    }
    let both = format!("{page}&ew=true");
    assert!(!in_project(&u(&both), dir));
}

#[test]
fn editor_stays_in_a_unix_project() {
    use crate::kit::Os;
    let u = |s: &str| s.parse::<Url>().unwrap();
    let in_project = |url: &Url, dir: &Path| in_project_on(Os::Linux, url, dir);
    let dir = Path::new("/home/me/Kit/projects/app & co");
    let page = crate::editor::url_on(Os::Linux, 7003, "tok", Some(dir));
    assert!(page.contains("folder=/home/"), "{page}");
    // macOS too: the same URL, matched exactly
    let mac = Path::new("/Users/me/Kit/projects/già");
    let mac_page = crate::editor::url_on(Os::Darwin, 7003, "tok", Some(mac));
    assert!(mac_page.contains("folder=/Users/"), "{mac_page}");
    assert!(in_project_on(Os::Darwin, &u(&mac_page), mac));
    assert!(in_project(&u(&page), dir));
    assert!(in_project(
        &u("http://127.0.0.1:7003/?folder=/home/me/Kit/projects/app%20%26%20co/"),
        dir
    ));
    for out in [
        // one more slash is another path for the server
        "http://127.0.0.1:7003/?folder=//home/me/Kit/projects/app%20%26%20co",
        // case matters on Linux
        "http://127.0.0.1:7003/?folder=/home/me/kit/projects/app%20%26%20co",
        "http://127.0.0.1:7003/?folder=/home/me/Kit/projects",
        "http://127.0.0.1:7003/?folder=/home/me/Kit/projects/app%20%26%20co/sub",
    ] {
        assert!(!in_project(&u(out), dir), "{out}");
    }
}

#[test]
fn title_bar_buttons() {
    let u = |s: &str| s.parse::<Url>().unwrap();
    let at = |p: &str| control(&u(&format!("http://127.0.0.1:7003{p}")), 7003);
    assert_eq!(at("/__atollo/window/minimize"), Some(Control::Minimize));
    assert_eq!(
        at("/__atollo/window/maximize"),
        Some(Control::ToggleMaximize)
    );
    assert_eq!(at("/__atollo/window/close"), Some(Control::Close));
    assert_eq!(at("/__atollo/window/close/x"), None);
    assert_eq!(at("/__atollo/window/"), None);
    assert_eq!(at("/?folder=/e:/p"), None);
    let other = u("http://127.0.0.1:7004/__atollo/window/close");
    assert_eq!(control(&other, 7003), None);
    let js = controls_script("en", 7003, "my app", &["a.b".into()]);
    assert!(js.contains(r#"close: "Close""#), "{js}");
    assert!(js.contains(r#"restart: "Restart extensions""#), "{js}");
    assert!(js.contains("window.__atolloExtensions = "), "{js}");
    assert!(js.contains(r#"c.productConfiguration = "#), "{js}");
    assert!(js.contains(r#"line("project", "my app")"#), "{js}");
    assert!(js.contains(r#"let ids = ["a.b"];"#), "{js}");
    assert!(
        js.contains(r#"localStorage.getItem("atollo-hidden:my app")"#),
        "{js}"
    );
    assert!(js.contains("data:image/png;base64,iVBOR"), "logo embedded");
    assert!(
        js.contains(r#"{"window.menuBarVisibility":"classic"}"#),
        "{js}"
    );
    assert!(
        js.contains(r#""http://127.0.0.1:7003/__atollo/window/" + what"#),
        "{js}"
    );
}

#[test]
fn origin_and_links() {
    let u = |s: &str| s.parse::<Url>().unwrap();
    assert!(same_origin(&u("http://127.0.0.1:7003/x?y"), 7003));
    assert!(!same_origin(&u("http://127.0.0.1:7004/"), 7003));
    assert!(!same_origin(&u("http://localhost:7003/"), 7003));
    assert!(!same_origin(&u("https://127.0.0.1:7003/"), 7003));
    assert_eq!(
        classify_external(&u("https://github.com/x")),
        Leave::Browser
    );
    assert_eq!(classify_external(&u("http://example.com")), Leave::Browser);
    for bad in [
        "file:///C:/Windows/win.ini",
        "javascript:alert(1)",
        "vscodium://x",
        "ms-settings:display",
    ] {
        assert_eq!(classify_external(&u(bad)), Leave::Block, "{bad}");
    }
}
