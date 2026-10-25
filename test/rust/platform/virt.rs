use super::*;

/// The check of this PC answers; a system Docker does not support is the
/// only gap listed.
#[test]
fn the_gaps_of_this_pc_are_read() {
    let gaps = gaps(None);
    let unsupported = |g: &Gap| {
        matches!(
            g,
            Gap::OldWindows(_) | Gap::Processor | Gap::OldMacos(_) | Gap::Distribution(_)
        )
    };
    if gaps.iter().any(unsupported) {
        assert_eq!(gaps.len(), 1, "{gaps:?}");
    }
}
