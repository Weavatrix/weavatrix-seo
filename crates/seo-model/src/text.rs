//! Text handling shared by every caller-supplied import.

/// Strips a UTF-8 byte-order mark.
///
/// Windows tooling writes one by default, and a JSON parser sees it as a
/// character before `{`. Rejecting an otherwise valid export with
/// "expected `{` at line 1 column 1" is a worse answer than reading it.
#[must_use]
pub fn strip_bom(raw: &str) -> &str {
    raw.strip_prefix('\u{feff}').unwrap_or(raw)
}

#[cfg(test)]
mod tests {
    use super::strip_bom;

    #[test]
    fn a_marked_document_parses_like_an_unmarked_one() {
        assert_eq!(strip_bom("\u{feff}{\"a\":1}"), "{\"a\":1}");
        assert_eq!(strip_bom("{\"a\":1}"), "{\"a\":1}");
    }

    #[test]
    fn only_a_leading_mark_is_stripped() {
        assert_eq!(strip_bom("{}\u{feff}"), "{}\u{feff}");
    }
}
