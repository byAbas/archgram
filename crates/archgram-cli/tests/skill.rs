//! The skill runs the archgram it was released with (docs/PRD.md 6.6):
//! every `archgram@X.Y.Z` in `skills/archgram` names this version, and no
//! command there runs archgram through npx without one.

use std::path::{Path, PathBuf};

fn markdown(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let path = entry.unwrap().path();
        if path.is_dir() {
            markdown(&path, found);
        } else if path.extension().is_some_and(|e| e == "md") {
            found.push(path);
        }
    }
}

#[test]
fn the_skill_names_the_version_it_is_released_with() {
    let skill = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/archgram");
    let version = env!("CARGO_PKG_VERSION");
    let mut files = Vec::new();
    markdown(&skill, &mut files);
    let mut pinned = 0;
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        for (n, line) in text.lines().enumerate() {
            let at = format!("{}:{}", file.display(), n + 1);
            for (i, _) in line.match_indices("archgram@") {
                let rest = &line[i + "archgram@".len()..];
                let end = rest
                    .find(|c: char| !c.is_ascii_digit() && c != '.')
                    .unwrap_or(rest.len());
                let named = rest[..end].trim_end_matches('.');
                assert_eq!(
                    named, version,
                    "{at} names archgram@{named}; this release is {version}"
                );
                pinned += 1;
            }
            // The one unpinned form is the permission to run the project's
            // own archgram, whose lockfile pins it.
            if !line.starts_with("allowed-tools:") {
                assert!(
                    !line.contains("npx --yes archgram "),
                    "{at} runs archgram through npx without a version: {line}"
                );
            }
        }
    }
    assert!(pinned > 0, "no archgram@X.Y.Z in {}", skill.display());
}
