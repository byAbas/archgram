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
/// plugin manifest reference puts every other file of a plugin at its root,
/// so the icon the manifest names for the directory's listing is there too.
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

    let manifest = std::fs::read_to_string(folder.join("plugin.json")).unwrap();
    let icon = manifest
        .lines()
        .find_map(|line| line.trim().strip_prefix("\"icon\": \"./"))
        .and_then(|rest| rest.split('"').next())
        .expect("plugin.json names an icon as `./<path>`");
    assert!(
        plugin.join(icon).is_file(),
        "plugin.json names the icon ./{icon}, which is not in {}",
        plugin.display()
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

/// Every file the skill names by its path from the skill's root, as the
/// Agent Skills format asks (`references/kinds/architecture/find.md`,
/// `scripts/size.mjs`), is there, so moving a file cannot leave the skill
/// pointing at nothing. A
/// path with a placeholder (`references/kinds/<kind>/find.md`) names no
/// one file.
#[test]
fn every_file_the_skill_names_is_there() {
    let skill = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/archgram");
    let mut files = Vec::new();
    markdown(&skill, &mut files);
    let mut named = 0;
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        for (n, line) in text.lines().enumerate() {
            let named_here = line
                .match_indices("`references/")
                .chain(line.match_indices("`scripts/"));
            for (i, _) in named_here {
                let path = line[i + 1..].split('`').next().unwrap_or_default();
                let file_like = Path::new(path).extension().is_some();
                if path.contains('<') || !file_like {
                    continue;
                }
                assert!(
                    skill.join(path).is_file(),
                    "{}:{} names {path}, which is not in the skill",
                    file.display(),
                    n + 1
                );
                named += 1;
            }
        }
    }
    assert!(named > 0, "the skill names no reference file");
}

/// Each kind's pack, `references/kinds/<kind>/`, is named after a kind
/// archgram draws, the value a spec's `diagram` takes, and holds the two
/// files the skill reads: `find.md` for the facts and `spec.md` for the
/// spec (docs/features/facts.md, Kind packs).
#[test]
fn every_kind_pack_is_a_kind_archgram_draws() {
    let kinds =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/archgram/references/kinds");
    let drawn: Vec<&str> = archgram_core::spec::DiagramKind::ALL
        .iter()
        .map(|k| k.name())
        .collect();
    let mut packs = 0;
    for entry in std::fs::read_dir(&kinds).unwrap_or_else(|e| panic!("{}: {e}", kinds.display())) {
        let path = entry.unwrap().path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        assert!(
            drawn.contains(&name.as_str()),
            "{} is a pack for `{name}`, which archgram does not draw ({drawn:?})",
            path.display()
        );
        for file in ["find.md", "spec.md"] {
            assert!(path.join(file).is_file(), "the {name} pack has no {file}");
        }
        // Its layer of the facts document (references/facts.md, A
        // diagram's layer), which its spec.md writes the spec from.
        let find = std::fs::read_to_string(path.join("find.md")).unwrap();
        assert!(
            find.contains("\n## In the facts document\n"),
            "the {name} pack's find.md does not say what its layer of the facts document holds"
        );
        packs += 1;
    }
    assert!(packs > 0, "{} holds no pack", kinds.display());
}

/// The skill downloads nothing the user has not agreed to (docs/PRD.md
/// 6.6): npx installs a missing package without asking when no terminal
/// is attached, so before its first `npx --yes` the skill looks for its
/// archgram with `--no --offline`, which uses no network and installs
/// nothing, and says it asks the user when it finds none.
#[test]
fn the_skill_looks_for_archgram_before_it_downloads() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/archgram/SKILL.md");
    let text = std::fs::read_to_string(&path).unwrap();
    let version = env!("CARGO_PKG_VERSION");
    let look = format!("npx --no --offline --loglevel=error archgram@{version} --version");
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
    let asks = &text[looks..downloads];
    assert!(
        asks.contains("ask the user before anything is"),
        "{}: between looking and the first download, the skill does not say it asks the user",
        path.display()
    );
}
