use super::*;

fn model(id: &str, files: &[&str], complete: bool, ctx: Option<u32>) -> LocalModel {
    LocalModel {
        id: id.into(),
        name: id.into(),
        repo: "o/r".into(),
        folder: "o--r".into(),
        files: files.iter().map(|f| (*f).to_owned()).collect(),
        mmproj: None,
        bytes: 1,
        downloaded: 0,
        complete,
        catalog: None,
        ctx,
        by_hand: false,
    }
}

/// One section per complete model: its first file, its projector, its
/// context (the catalog's, or the kit's default).
#[test]
fn a_section_per_complete_model() {
    let root = Path::new("/kit/data/models/gguf");
    let mut vision = model("v", &["v.gguf"], true, None);
    vision.mmproj = Some("mmproj-F16.gguf".into());
    let text = text(
        root,
        &[
            model(
                "coder",
                &["c-00001-of-00002.gguf", "c-00002-of-00002.gguf"],
                true,
                Some(32768),
            ),
            model("half", &["h.gguf"], false, None),
            vision,
        ],
    );
    let first = root.join("o--r").join("c-00001-of-00002.gguf");
    assert!(text.contains(&format!(
        "[coder]\nmodel = {}\nctx-size = 32768\n",
        first.display()
    )));
    assert!(!text.contains("[half]"));
    assert!(text.contains(&format!(
        "[v]\nmodel = {}\nmmproj = {}\nctx-size = {DEFAULT_CTX}\n",
        root.join("o--r").join("v.gguf").display(),
        root.join("o--r").join("mmproj-F16.gguf").display()
    )));
}
