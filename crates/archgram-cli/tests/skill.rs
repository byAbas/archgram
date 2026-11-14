//! The skill runs the archgram it was released with (docs/PRD.md 6.6):
//! every `archgram@X.Y.Z` in `skills/archgram` names this version, and no
//! command there runs archgram through npx without one.

use std::path::{Path, PathBuf};

/// How the skill runs archgram through npx. `--loglevel=error` keeps npm's
/// own warnings, such as those about a pnpm project's `.npmrc`, out of what
/// the agent reads (#52).
const NPX: &str = "npx --yes --loglevel=error archgram";

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
            assert!(
                !line.contains(&format!("{NPX} ")),
                "{at} runs archgram through npx without a version: {line}"
            );
            assert!(
                !line.contains("npx --yes archgram"),
                "{at} runs archgram through npx without `--loglevel=error`: {line}"
            );
        }
    }
    assert!(pinned > 0, "no archgram@X.Y.Z in {}", skill.display());
}

/// The skill's folder is also a Claude Code plugin (docs/PRD.md 6.5), whose
/// manifest carries the release's version, so an update reaches its users.
#[test]
fn the_plugin_carries_the_version_it_is_released_with() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../skills/archgram/.claude-plugin/plugin.json");
    let text = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|e| panic!("{}: {e}", manifest.display()));
    let version = env!("CARGO_PKG_VERSION");
    assert!(
        text.contains(&format!("\"version\": \"{version}\"")),
        "{} does not carry \"version\": \"{version}\"",
        manifest.display()
    );
}

/// The skill pre-approves no command: its frontmatter has no `allowed-tools`.
/// Anthropic's plugin directory accepts an entry beyond Read, Grep, Glob and
/// LS only from a Verified organization, and refused a version whose skill
/// pre-approved its own `archgram spec` commands. Every command asks the user.
#[test]
fn the_skill_preapproves_no_command() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/archgram/SKILL.md");
    let text = std::fs::read_to_string(&path).unwrap();
    let (frontmatter, _) = text
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .expect("SKILL.md starts with a frontmatter between `---` lines");
    for (n, line) in frontmatter.lines().enumerate() {
        assert!(
            !line.starts_with("allowed-tools:") && !line.starts_with("allowedTools:"),
            "{}:{} pre-approves commands; the directory refuses that",
            path.display(),
            n + 2
        );
    }
}

/// The frontmatter is YAML that any parser reads: a value written without
/// quotes holds no `: ` and no ` #`, which YAML reads as a new key and a
/// comment. Claude Code reads such a line anyway, but Anthropic's plugin
/// directory could not, and held the version (`REACH_UNKNOWN`).
#[test]
fn the_skill_frontmatter_is_plain_yaml() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/archgram/SKILL.md");
    let text = std::fs::read_to_string(&path).unwrap();
    let frontmatter = text
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map(|(frontmatter, _)| frontmatter)
        .expect("SKILL.md starts with a frontmatter between `---` lines");
    for (n, line) in frontmatter.lines().enumerate() {
        let at = format!("{}:{}", path.display(), n + 2);
        let (key, value) = line
            .split_once(": ")
            .unwrap_or_else(|| panic!("{at} is not `key: value`: {line}"));
        assert!(
            !value.starts_with(['"', '\'', '|', '>']),
            "{at}: `{key}` is quoted or a block; keep it plain"
        );
        for breaks in [": ", " #"] {
            assert!(
                !value.contains(breaks),
                "{at}: `{key}` holds `{breaks}`, which ends a plain YAML value"
            );
        }
    }
}
