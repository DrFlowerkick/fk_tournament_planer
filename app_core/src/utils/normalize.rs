/// Normalize whitespace by:
/// - mapping all Unicode whitespace (incl. tabs/newlines) to a single ASCII space
/// - collapsing runs of spaces to a single space
/// - trimming leading/trailing spaces
pub fn normalize_ws(input: impl Into<String>) -> String {
    // Map all whitespace chars to ' ' and keep others unchanged
    let mapped: String = input
        .into()
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();

    // Collapse runs of ' ' to a single space
    let mut out = String::with_capacity(mapped.len());
    let mut last_space = false;
    for ch in mapped.chars() {
        if ch == ' ' {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else {
            out.push(ch);
            last_space = false;
        }
    }

    out.trim().to_string()
}

/// Normalize an optional string:
/// - apply whitespace normalization to Some
/// - convert empty result to None
pub fn normalize_opt(input: Option<impl Into<String>>) -> Option<String> {
    match input {
        None => None,
        Some(s) => {
            let n = normalize_ws(s);
            if n.is_empty() { None } else { Some(n) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // normalize_ws
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn ws_trims_leading_and_trailing_spaces() {
        assert_eq!(normalize_ws("  hello  "), "hello");
        assert_eq!(normalize_ws("\t\t hello\n"), "hello");
    }

    #[test]
    fn ws_collapses_internal_whitespace_runs() {
        assert_eq!(normalize_ws("a   b    c"), "a b c");
        assert_eq!(normalize_ws("a\tb\t\tc"), "a b c");
        assert_eq!(normalize_ws("a\n\nb\nc"), "a b c");
    }

    #[test]
    fn ws_maps_unicode_whitespace_to_ascii_space() {
        // NBSP (U+00A0), EN SPACE (U+2002), EM SPACE (U+2003), THIN SPACE (U+2009)
        let input = "\u{00A0}foo\u{2002}\u{2003}bar\u{2009}baz\u{00A0}";
        let out = normalize_ws(input);
        assert_eq!(out, "foo bar baz");
    }

    #[test]
    fn ws_is_idempotent() {
        let once = normalize_ws("  a   \n  b\t\tc  ");
        let twice = normalize_ws(&once);
        assert_eq!(
            once, twice,
            "normalize_ws(normalize_ws(x)) must equal normalize_ws(x)"
        );
    }

    #[test]
    fn ws_preserves_non_whitespace_chars() {
        // Make sure we don't strip or alter non-space characters.
        let input = "Straße \u{212B} 𝛂 / №42 – Café";
        let out = normalize_ws(input);
        assert_eq!(out, "Straße \u{212B} 𝛂 / №42 – Café");
    }

    #[test]
    fn ws_empty_and_whitespace_only_become_empty_string() {
        assert_eq!(normalize_ws(""), "");
        assert_eq!(normalize_ws("   "), "");
        assert_eq!(normalize_ws("\n\t\u{00A0}"), "");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // normalize_opt
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn opt_none_stays_none() {
        let v: Option<String> = None;
        assert_eq!(normalize_opt(v), None);
    }

    #[test]
    fn opt_some_with_content_is_trimmed_and_collapsed() {
        let v = Some("  Main   Campus  ");
        let n = normalize_opt(v);
        assert_eq!(n.as_deref(), Some("Main Campus"));
    }

    #[test]
    fn opt_some_whitespace_only_becomes_none() {
        for s in [" ", "\t\t", "\n", " \u{00A0} \u{2003} "] {
            let v = Some(s);
            let n = normalize_opt(v);
            assert_eq!(n, None, "whitespace-only should normalize to None");
        }
    }

    #[test]
    fn opt_is_idempotent() {
        let v = Some("  a   b  ");
        let once = normalize_opt(v);
        let twice = normalize_opt(once.as_ref());
        assert_eq!(once, twice);
    }
}
