//! The skills run the archgram they were released with (docs/PRD.md 6.6):
//! every `archgram@X.Y.Z` in `skills/archgram` names this version, and no
//! command there runs archgram through npx without one. One skill for each
//! kind of diagram, `skills/archgram/skills/draw-<kind>-diagram/`, each
//! small enough to read whole.

use std::path::{Path, PathBuf};

/// How the skill runs archgram through npx. `--loglevel=error` keeps npm's
/// own warnings, such as those about a pnpm project's `.npmrc`, out of what
/// the agent reads (#52).
const NPX: &str = "npx --yes --loglevel=error archgram";

/// Each skill's folder, `skills/archgram/skills/<name>/`.
fn skills() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/archgram/skills");
    let mut found: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("{}: {e}", root.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    found.sort();
    assert!(!found.is_empty(), "{} holds no skill", root.display());
    found
}

/// A skill's `SKILL.md` frontmatter, between its `---` lines.
fn frontmatter(path: &Path) -> String {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    text.strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map_or_else(
            || {
                panic!(
                    "{} starts with no frontmatter between `---` lines",
                    path.display()
                )
            },
            |(frontmatter, _)| frontmatter.to_owned(),
        )
}

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

/// The skills' folder is also a Claude Code plugin (docs/PRD.md 6.5), whose
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

/// The repository is also a Claude Code marketplace of that one plugin
/// (docs/PRD.md 6.5): its entry is the skill's folder by relative path, so
/// Claude Code copies that folder alone, and it sets no version of its own,
/// so the manifest's, the release's, is the one users get.
#[test]
fn the_marketplace_lists_the_skill_folder_as_its_one_plugin() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root.join(".claude-plugin/marketplace.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert!(
        text.contains("\"source\": \"./skills/archgram\""),
        "{} does not list ./skills/archgram",
        path.display()
    );
    assert_eq!(
        text.matches("\"source\":").count(),
        1,
        "{} lists more than one plugin",
        path.display()
    );
    assert!(
        !text.contains("\"version\""),
        "{} sets a version; the plugin's manifest carries it",
        path.display()
    );
    assert!(
        root.join("skills/archgram/.claude-plugin/plugin.json")
            .is_file(),
        "the folder the marketplace lists has no plugin manifest"
    );
}

/// `.claude-plugin/` holds the manifest and nothing else: Claude Code's
/// plugin manifest reference puts every other file of a plugin at its root.
/// Finder's `.DS_Store` is ignored by git, so it is never submitted.
#[test]
fn the_plugin_keeps_only_its_manifest_in_the_manifest_folder() {
    let plugin = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/archgram");
    let folder = plugin.join(".claude-plugin");
    for entry in std::fs::read_dir(&folder).unwrap_or_else(|e| panic!("{}: {e}", folder.display()))
    {
        let name = entry.unwrap().file_name();
        let name = name.to_string_lossy();
        assert!(
            name == "plugin.json" || name == ".DS_Store",
            "{} holds {name}; keep it at the plugin's root",
            folder.display()
        );
    }
}

/// The manifest names none of the fields only Anthropic's directory reads.
/// The directory took the listing's icon and links when the submission was
/// created, and newer versions do not update them; each field left in a
/// version only raises a warning in its scan, as one Claude Code does not
/// read.
#[test]
fn the_plugin_manifest_has_no_directory_listing_fields() {
    let manifest = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../skills/archgram/.claude-plugin/plugin.json"),
    )
    .unwrap();
    for field in [
        "icon",
        "documentationUrl",
        "supportUrl",
        "privacyPolicyUrl",
        "termsOfServiceUrl",
    ] {
        assert!(
            !manifest.contains(&format!("\"{field}\"")),
            "plugin.json names `{field}`, which only the directory's listing reads"
        );
    }
}

/// The skill pre-approves no command: its frontmatter has no `allowed-tools`.
/// Anthropic's plugin directory accepts an entry beyond Read, Grep, Glob and
/// LS only from a Verified organization, and refused a version whose skill
/// pre-approved its own `archgram spec` commands. Every command asks the user.
#[test]
fn the_skill_preapproves_no_command() {
    for skill in skills() {
        let path = skill.join("SKILL.md");
        for (n, line) in frontmatter(&path).lines().enumerate() {
            assert!(
                !line.starts_with("allowed-tools:") && !line.starts_with("allowedTools:"),
                "{}:{} pre-approves commands; the directory refuses that",
                path.display(),
                n + 2
            );
        }
    }
}

/// The frontmatter is YAML that any parser reads: a value written without
/// quotes holds no `: ` and no ` #`, which YAML reads as a new key and a
/// comment. Claude Code reads such a line anyway, but Anthropic's plugin
/// directory could not, and held the version (`REACH_UNKNOWN`).
#[test]
fn the_skill_frontmatter_is_plain_yaml() {
    for skill in skills() {
        let path = skill.join("SKILL.md");
        for (n, line) in frontmatter(&path).lines().enumerate() {
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
}

/// Every file a skill names in its own folder (`find.md`, `spec.md`) is
/// there, so moving a file cannot leave a skill pointing at nothing; it
/// names them one level deep, as the Agent Skills format recommends.
#[test]
fn every_file_a_skill_names_is_there() {
    let mut named = 0;
    for skill in skills() {
        let mut files = Vec::new();
        markdown(&skill, &mut files);
        for file in &files {
            let text = std::fs::read_to_string(file).unwrap();
            for (n, line) in text.lines().enumerate() {
                for part in line.split('`').skip(1).step_by(2) {
                    let own = Path::new(part)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                        && !part.contains(['/', '<', ' '])
                        && part != "SKILL.md";
                    if !own {
                        continue;
                    }
                    assert!(
                        skill.join(part).is_file(),
                        "{}:{} names {part}, which is not in {}",
                        file.display(),
                        n + 1,
                        skill.display()
                    );
                    named += 1;
                }
            }
        }
    }
    assert!(named > 0, "no skill names a file of its own");
}

/// Each skill is named `draw-<kind>-diagram` after a kind archgram draws,
/// the value a spec's `diagram` takes, its frontmatter's `name` its
/// folder's, and holds `SKILL.md` and `find.md`, what its kind draws and
/// how to read the project for it (docs/PRD.md 6.6).
#[test]
fn every_skill_is_a_kind_archgram_draws() {
    let drawn: Vec<&str> = archgram_core::spec::DiagramKind::ALL
        .iter()
        .map(|k| k.name())
        .collect();
    for skill in skills() {
        let name = skill.file_name().unwrap().to_string_lossy().into_owned();
        let kind = name
            .strip_prefix("draw-")
            .and_then(|rest| rest.strip_suffix("-diagram"))
            .unwrap_or_else(|| panic!("{} is not named draw-<kind>-diagram", skill.display()));
        assert!(
            drawn.contains(&kind),
            "{} draws `{kind}`, which archgram does not draw ({drawn:?})",
            skill.display()
        );
        assert!(
            frontmatter(&skill.join("SKILL.md")).contains(&format!("\nname: {name}\n"))
                || frontmatter(&skill.join("SKILL.md")).starts_with(&format!("name: {name}\n")),
            "{}/SKILL.md is not named {name}",
            skill.display()
        );
        for file in ["SKILL.md", "find.md"] {
            assert!(skill.join(file).is_file(), "{name} has no {file}");
        }
    }
}

/// Each `SKILL.md` stays small enough to read whole: under 500 lines and
/// about 5,000 tokens, counted as 20,000 bytes, as the Agent Skills format
/// recommends (docs/PRD.md 6.6).
#[test]
fn every_skill_is_small_enough_to_read_whole() {
    for skill in skills() {
        let path = skill.join("SKILL.md");
        let text = std::fs::read_to_string(&path).unwrap();
        let lines = text.lines().count();
        assert!(
            lines < 500,
            "{} is {lines} lines; keep it under 500",
            path.display()
        );
        assert!(
            text.len() < 20_000,
            "{} is {} bytes, more than about 5,000 tokens",
            path.display(),
            text.len()
        );
    }
}

/// The skill downloads nothing the user has not agreed to (docs/PRD.md
/// 6.6): npx installs a missing package without asking when no terminal
/// is attached, so before its first `npx --yes` the skill looks for its
/// archgram with `--no --offline`, which uses no network and installs
/// nothing, and says it asks the user when it finds none.
#[test]
fn the_skill_looks_for_archgram_before_it_downloads() {
    let version = env!("CARGO_PKG_VERSION");
    let look = format!("npx --no --offline --loglevel=error archgram@{version} --version");
    for skill in skills() {
        let path = skill.join("SKILL.md");
        let text = std::fs::read_to_string(&path).unwrap();
        let looks = text
            .find(&look)
            .unwrap_or_else(|| panic!("{} never runs `{look}`", path.display()));
        let downloads = text
            .find("npx --yes")
            .unwrap_or_else(|| panic!("{} runs no `npx --yes`", path.display()));
        assert!(
            looks < downloads,
            "{} runs `npx --yes` before it looks for archgram with `{look}`",
            path.display()
        );
        assert!(
            text[looks..downloads].contains("ask before downloading"),
            "{}: between looking and the first download, the skill does not say it asks",
            path.display()
        );
    }
}

/// The plugin's folder holds only text: Markdown, text, JSON and YAML, none
/// starting with `#!`. Anthropic's directory accepts no script from an
/// organization that is not Verified, and notes each image a version
/// carries as let through without being read; the listing's icon is the one
/// uploaded in the developer portal, kept as `docs/images/plugin-icon.png`.
#[test]
fn the_plugin_holds_only_text() {
    // What git tracks, as the directory reads the commit, not the folder
    // (a `.DS_Store` beside it is no file of the plugin's).
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let listed = std::process::Command::new("git")
        .args(["ls-files", "-z", "--", "skills/archgram"])
        .current_dir(&root)
        .output()
        .expect("git runs");
    let files: Vec<PathBuf> = listed
        .stdout
        .split(|&b| b == 0)
        .filter(|f| !f.is_empty())
        .map(|f| root.join(String::from_utf8_lossy(f).as_ref()))
        .collect();
    assert!(!files.is_empty(), "git lists the skill's files");
    for file in &files {
        let extension = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        assert!(
            ["md", "txt", "json", "yaml", "yml"].contains(&extension.as_str()),
            "{} is not a file the directory accepts in the plugin",
            file.display()
        );
        let head = std::fs::read(file).unwrap();
        assert!(
            !head.starts_with(b"#!"),
            "{} starts with #!",
            file.display()
        );
    }
}
