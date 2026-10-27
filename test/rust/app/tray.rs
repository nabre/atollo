use super::*;

fn ids(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

/// The projects keep the order they come in (the most recent first) and
/// each id leads back to its project.
#[test]
fn projects_in_order_and_back_from_their_ids() {
    let s = spec("it", &ids(&["recent", "older", "a&b"]));
    let labels: Vec<&str> = s.entries.iter().map(|(_, l)| l.as_str()).collect();
    assert_eq!(labels, ["recent", "older", "a&&b"]);
    let back: Vec<Option<Action>> = s.entries.iter().map(|(id, _)| action(id)).collect();
    assert_eq!(
        back,
        [
            Some(Action::Open("recent".into())),
            Some(Action::Open("older".into())),
            Some(Action::Open("a&b".into())),
        ]
    );
}

#[test]
fn at_most_max_projects() {
    let many: Vec<String> = (0..MAX_PROJECTS + 5).map(|i| format!("p{i}")).collect();
    let s = spec("en", &many);
    assert_eq!(s.entries.len(), MAX_PROJECTS);
    assert_eq!(s.entries[0].1, "p0");
}

/// No projects: the submenu has its disabled item only, which does nothing.
#[test]
fn no_projects() {
    let s = spec("en", &[]);
    assert!(s.entries.is_empty());
    assert_eq!(s.empty, "No projects");
    assert_eq!(action(NONE), None);
    assert_eq!(action(PROJECT), None);
    assert_eq!(action("other"), None);
}

#[test]
fn show_and_quit() {
    assert_eq!(action(SHOW), Some(Action::Show));
    assert_eq!(action(QUIT), Some(Action::Quit));
}

/// The texts follow the app's language; a project named like a menu id
/// stays a project.
#[test]
fn texts_in_every_language() {
    let it = spec("it", &[]);
    assert_eq!(
        (it.show.as_str(), it.projects.as_str(), it.quit.as_str()),
        ("Mostra Atollo", "Progetti", "Chiudi Atollo")
    );
    assert_eq!(spec("en", &[]).quit, "Close Atollo");
    assert_eq!(spec("fr", &[]).quit, "Fermer Atollo");
    assert_eq!(spec("de", &[]).quit, "Atollo schließen");
    let s = spec("en", &ids(&["show"]));
    assert_eq!(action(&s.entries[0].0), Some(Action::Open("show".into())));
}

/// The label shows `&` as it is on every platform (muda's mnemonics).
#[test]
fn ampersands_are_doubled() {
    assert_eq!(label("R&D && more"), "R&&D &&&& more");
    assert_eq!(label("plain_name"), "plain_name");
}

/// One reading of the projects at a time: a refresh asked while one reads
/// makes it read once more (however many were asked), none is lost, and
/// one asked after the reading ended starts its own.
#[test]
fn refreshes_asked_meanwhile_make_one_more_reading() {
    let (reading, again) = (AtomicBool::new(false), AtomicBool::new(false));
    assert!(start_reading(&reading, &again), "the first reads");
    assert!(!start_reading(&reading, &again), "meanwhile: folded");
    assert!(!start_reading(&reading, &again), "meanwhile: folded");
    assert!(read_again(&reading, &again), "one more for those");
    // That reading starts by clearing `again`; nothing asked meanwhile.
    again.store(false, Ordering::SeqCst);
    assert!(!read_again(&reading, &again), "nothing asked: done");
    assert!(!reading.load(Ordering::SeqCst));
    assert!(start_reading(&reading, &again), "a new one reads");
}
