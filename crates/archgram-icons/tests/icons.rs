//! The logo data: every line well formed, slugs sorted and unique, and the
//! logos the examples name are there.

use archgram_icons::Icons;

#[test]
fn the_data_is_sorted_unique_and_whole() {
    let icons = Icons::load();
    assert!(icons.len() > 3000, "{} logos", icons.len());
    let slugs: Vec<&str> = archgram_core::logos::Logos::slugs(&icons);
    assert!(
        slugs.windows(2).all(|w| w[0] < w[1]),
        "slugs sorted and unique"
    );
    for slug in &slugs {
        let icon = icons.get(slug).unwrap();
        assert!(
            !icon.title.is_empty() && icon.path.starts_with(['M', 'm']),
            "{slug}"
        );
        assert!(
            icon.hex.len() == 6 && icon.hex.chars().all(|c| c.is_ascii_hexdigit()),
            "{slug}: {}",
            icon.hex
        );
    }
}

#[test]
fn the_examples_logos_are_carried() {
    let icons = Icons::load();
    for slug in ["fastapi", "postgresql", "redis", "googlegemini"] {
        assert!(icons.get(slug).is_some(), "{slug}");
    }
    assert!(icons.get("no-such-logo").is_none());
}

#[test]
fn a_wrong_slug_is_refused_with_the_nearest() {
    let spec = archgram_core::parse_spec(
        r#"{ "archgram": 1, "title": "t", "description": "d",
        "nodes": [{ "id": "db", "kind": "database", "label": "DB", "tech": "postgressql" }] }"#,
    )
    .unwrap();
    let errors = archgram_core::check_logos(&spec, &Icons::load());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].location.to_string(), "/nodes/0/tech");
    assert!(
        errors[0].message.contains("`postgresql`"),
        "{}",
        errors[0].message
    );
}

/// Every logo is carried, whatever its licence, and the crate's licence
/// names every licence a logo carries (`data/provenance.tsv`), so what it
/// declares is what it ships (#176).
#[test]
fn every_logo_is_carried_and_its_licence_declared() {
    let icons = Icons::load();
    for slug in [
        "javascript",
        "rust",
        "openjdk",
        "gnubash",
        "php",
        "vuedotjs",
    ] {
        assert!(icons.get(slug).is_some(), "{slug}");
    }
    let declared: Vec<&str> = env!("CARGO_PKG_LICENSE").split(" AND ").collect();
    let provenance = include_str!("../data/provenance.tsv");
    let mut rows = 0;
    for line in provenance.lines() {
        rows += 1;
        let licence = line.split('\t').nth(3).expect("a licence column");
        let spdx = match licence {
            "" => "CC0-1.0",
            "custom" => "LicenseRef-SimpleIcons-Custom",
            other => other,
        };
        assert!(declared.contains(&spdx), "{line}: {spdx} not declared");
    }
    assert_eq!(rows, icons.len(), "one provenance row for each logo");
}
