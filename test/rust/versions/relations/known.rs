use super::*;
use crate::setup::Catalog;
use crate::versions::ranges::{Bare, Range};
use crate::versions::relations::split;

#[test]
fn knowledge_is_valid() {
    let k: Known = serde_json::from_str(KNOWN).unwrap();
    let catalog = Catalog::get();
    let parts = ["composer", "gradle", "maven"];
    for l in &k.links {
        let (c, part) = split(&l.from);
        assert!(catalog.find(c).is_some(), "{}", l.from);
        assert!(part.is_none_or(|p| parts.contains(&p)), "{}", l.from);
        assert!(catalog.find(&l.to).is_some(), "{}", l.to);
        if let Some(r) = &l.range {
            assert!(Range::parse(r, Bare::Exact).is_some(), "{r}");
        }
        assert!(["needs", "uses", "runs-on", "loads"].contains(&l.kind.as_str()));
    }
    for r in &k.runtimes {
        for f in &r.for_ {
            assert!(f.starts_with('@') || catalog.find(f).is_some(), "{f}");
        }
    }
    for p in &k.projects {
        assert!(catalog.find(&p.component).is_some(), "{}", p.component);
        assert!(
            p.json.is_some() || p.regex.is_some() || p.whole,
            "{}",
            p.file
        );
        if let Some(re) = &p.regex {
            regex::Regex::new(re).unwrap();
        }
    }
}
