// Accept explicit registry selectors; never infer registry routing by excluding prefixes.
pub(super) fn registry_specifier(value: &str) -> bool {
    if let Some(alias) = value.strip_prefix("npm:") {
        let Some((package, selector)) = alias.rsplit_once('@') else {
            return false;
        };
        let parts: Vec<_> = package.trim_start_matches('@').split('/').collect();
        let valid_name = |part: &&str| {
            !part.is_empty()
                && !part.starts_with(['.', '_'])
                && part.bytes().all(|b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_' | b'.')
                })
        };
        return parts.len() == if package.starts_with('@') { 2 } else { 1 }
            && parts.iter().all(valid_name)
            && registry_selector(selector);
    }
    registry_selector(value)
}

fn registry_selector(value: &str) -> bool {
    if value.is_empty()
        || value.trim() != value
        || value.len() > 256
        || value.bytes().any(|b| b.is_ascii_control())
    {
        return false;
    }
    if value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        && value.bytes().any(|b| b.is_ascii_alphabetic())
    {
        return true;
    }
    value.split("||").all(|branch| {
        let branch = branch.trim();
        if branch.is_empty() {
            return false;
        }
        let range = if let Some((left, right)) = branch.split_once(" - ") {
            format!(">={}, <= {}", left.trim(), right.trim())
        } else {
            let mut tokens: Vec<String> = Vec::new();
            for token in branch.split_whitespace() {
                if tokens.last().is_some_and(|last| {
                    matches!(last.as_str(), ">" | ">=" | "<" | "<=" | "=" | "~" | "^")
                }) {
                    tokens.last_mut().expect("checked token").push_str(token);
                } else {
                    tokens.push(token.to_owned());
                }
            }
            tokens.join(", ")
        };
        semver::VersionReq::parse(&range.replace(['x', 'X'], "*")).is_ok()
    })
}

#[cfg(test)]
mod specifier_tests {
    use super::registry_specifier;
    #[test]
    fn classifies_registry_versions_tags_aliases_and_external_routes() {
        for value in [
            "1.2.3",
            "^1.2",
            ">=1 <2",
            "1.x",
            "*",
            "1.2.3 - 2.3.4",
            "^1 || ^2",
            "latest",
            "beta.1",
            "npm:express@^5",
            "npm:@scope/name@1.0.0",
        ] {
            assert!(registry_specifier(value), "rejected {value}");
        }
        for value in [
            "expressjs/express",
            "github:expressjs/express",
            "git+https://github.com/a/b",
            "https://example.org/a.tgz",
            "file:../a",
            "link:a",
            "workspace:*",
            "../a",
            "/a",
            "npm:express@github:a/b",
            "npm:express",
            "",
            "1 ||",
            "1\n2",
            "latest ",
        ] {
            assert!(!registry_specifier(value), "accepted {value}");
        }
    }
}
