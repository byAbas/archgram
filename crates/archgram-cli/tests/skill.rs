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
            assert!(
                !line.contains("npx --yes archgram "),
                "{at} runs archgram through npx without a version: {line}"
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

/// `allowed-tools` pre-approves exactly the archgram commands the skill
/// runs, at its version, and nothing broader, as Anthropic's plugin
/// directory asks: every command is covered, and every rule is used.
#[test]
fn the_skill_preapproves_only_the_commands_it_runs() {
    let skill = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/archgram");
    let mut files = Vec::new();
    markdown(&skill, &mut files);
    let prefix = format!("npx --yes archgram@{} ", env!("CARGO_PKG_VERSION"));
    let mut commands = Vec::new();
    for file in &files {
        for line in std::fs::read_to_string(file).unwrap().lines() {
            if line.starts_with("allowed-tools:") {
                continue;
            }
            for (i, _) in line.match_indices(&prefix) {
                let rest = &line[i..];
                let end = rest.find('`').unwrap_or(rest.len());
                commands.push(rest[..end].trim_end().to_string());
            }
        }
    }
    assert!(!commands.is_empty(), "no `{prefix}…` command in the skill");

    let manifest = std::fs::read_to_string(skill.join("SKILL.md")).unwrap();
    let allowed = manifest
        .lines()
        .find_map(|line| line.strip_prefix("allowed-tools: "))
        .expect("SKILL.md has allowed-tools");
    let rules: Vec<&str> = allowed
        .split(") ")
        .map(|rule| rule.trim_start_matches("Bash(").trim_end_matches(')'))
        .collect();
    let covers = |rule: &str, command: &str| match rule.strip_suffix(" *") {
        Some(head) => command == head || command.starts_with(&format!("{head} ")),
        None => command == rule,
    };
    for rule in &rules {
        assert!(
            rule.starts_with(&prefix) && rule.len() > prefix.len() + 1,
            "allowed-tools rule `{rule}` is not one archgram command at this version"
        );
        assert!(
            commands.iter().any(|command| covers(rule, command)),
            "allowed-tools rule `{rule}` covers no command the skill runs"
        );
    }
    for command in &commands {
        assert!(
            rules.iter().any(|rule| covers(rule, command)),
            "`{command}` is not pre-approved in allowed-tools"
        );
    }
}
