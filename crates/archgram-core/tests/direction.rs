//! `direction: auto` (docs/SPEC.md, Top level): left to right while the
//! drawing fits where it is shown ([`readable_width`]: [`README_WIDTH`] in
//! a README), top to bottom when it would not, and the same bytes as naming
//! that direction outright.

use archgram_core::render::Options;
use archgram_core::spec::Direction;
use archgram_core::{
    Drawing, README_WIDTH, draw_themes, draw_with, logos::NoLogos, parse_spec, readable_width,
};

/// A chain of `n` services, each calling the next: the longer the chain,
/// the wider it is drawn left to right.
fn chain(n: usize, direction: &str) -> String {
    let nodes: Vec<String> = (0..n)
        .map(|i| format!(r#"{{ "id": "s{i}", "kind": "service", "label": "Service {i}" }}"#))
        .collect();
    let edges: Vec<String> = (1..n)
        .map(|i| format!(r#"{{ "from": "s{}", "to": "s{i}" }}"#, i - 1))
        .collect();
    format!(
        r#"{{ "archgram": 1, "title": "chain", "description": "A chain of services.",
             "direction": "{direction}", "nodes": [{}], "edges": [{}] }}"#,
        nodes.join(", "),
        edges.join(", ")
    )
}

fn drawn(json: &str) -> Drawing {
    draw_with(&parse_spec(json).unwrap(), Options::default(), &NoLogos).unwrap()
}

#[test]
fn auto_keeps_left_to_right_while_it_fits() {
    let auto = drawn(&chain(3, "auto"));
    let right = drawn(&chain(3, "right"));
    assert!(
        right.width <= README_WIDTH,
        "a short chain fits: {}",
        right.width
    );
    assert_eq!(auto.direction, Direction::Right);
    assert_eq!(
        auto,
        Drawing {
            direction: Direction::Right,
            ..right
        }
    );
}

#[test]
fn auto_lays_out_top_to_bottom_when_left_to_right_is_too_wide() {
    let auto = drawn(&chain(9, "auto"));
    let right = drawn(&chain(9, "right"));
    let down = drawn(&chain(9, "down"));
    assert!(
        right.width > README_WIDTH,
        "a long chain is too wide: {}",
        right.width
    );
    assert_eq!(auto.direction, Direction::Down);
    assert_eq!(auto.svg, down.svg, "the same bytes as naming `down`");
    assert!(auto.width < right.width);
}

#[test]
fn a_named_direction_is_kept_however_wide() {
    let right = drawn(&chain(9, "right"));
    assert_eq!(right.direction, Direction::Right);
    assert!(right.width > README_WIDTH);
}

#[test]
fn the_size_is_the_svgs_own() {
    for n in [3, 9] {
        let d = drawn(&chain(n, "auto"));
        let head = d.svg.lines().next().unwrap();
        let width = format!(r#" width="{}""#, d.width);
        assert!(head.contains(&width), "{head} has {width}");
    }
}

#[test]
fn both_themes_take_the_same_direction_and_size() {
    let spec = parse_spec(&chain(9, "auto")).unwrap();
    let (light, dark) = draw_themes(&spec, Options::default(), &NoLogos).unwrap();
    assert_eq!(light.direction, Direction::Down);
    assert_eq!(
        (light.width, light.height, light.direction),
        (dark.width, dark.height, dark.direction)
    );
}

#[test]
fn auto_draws_the_same_bytes_every_time() {
    assert_eq!(drawn(&chain(9, "auto")).svg, drawn(&chain(9, "auto")).svg);
}

/// A binary tree of `n` services: left to right it is tall and fairly
/// narrow, top to bottom very wide.
fn tree(n: usize, direction: &str) -> String {
    let nodes: Vec<String> = (0..n)
        .map(|i| format!(r#"{{ "id": "n{i}", "kind": "service", "label": "Service {i}" }}"#))
        .collect();
    let edges: Vec<String> = (0..n)
        .flat_map(|i| {
            [2 * i + 1, 2 * i + 2]
                .into_iter()
                .filter(move |&j| j < n)
                .map(move |j| (i, j))
        })
        .map(|(i, j)| format!(r#"{{ "from": "n{i}", "to": "n{j}" }}"#))
        .collect();
    format!(
        r#"{{ "archgram": 1, "title": "tree", "description": "A tree of services.",
             "direction": "{direction}", "nodes": [{}], "edges": [{}] }}"#,
        nodes.join(", "),
        edges.join(", ")
    )
}

#[test]
fn auto_keeps_the_narrower_direction_when_neither_fits() {
    let right = drawn(&tree(100, "right"));
    let down = drawn(&tree(100, "down"));
    assert!(
        right.width > README_WIDTH && down.width > right.width,
        "{} {}",
        right.width,
        down.width
    );
    let auto = drawn(&tree(100, "auto"));
    assert_eq!(auto.direction, Direction::Right);
    assert_eq!(auto.svg, right.svg);
}

/// Where a diagram is shown sets the width `auto` keeps within: the README's
/// 1,300 px by default, and the same ratio of `shownWidth` elsewhere.
#[test]
fn readable_width_is_the_readme_s_ratio_of_where_it_is_shown() {
    let spec = |rest: &str| {
        parse_spec(&chain(2, "auto").replacen(
            r#""direction""#,
            &format!(r#"{rest} "direction""#),
            1,
        ))
        .unwrap()
    };
    assert!((readable_width(&spec("")) - README_WIDTH).abs() < f64::EPSILON);
    assert!((readable_width(&spec(r#""shownWidth": 880,"#)) - README_WIDTH).abs() < 1e-9);
    let column = readable_width(&spec(r#""shownWidth": 674,"#));
    assert!((column - 674.0 * 1300.0 / 880.0).abs() < 1e-9, "{column}");
}

/// A chain that fits a README left to right but not a 674 px column turns
/// top to bottom there, and only there.
#[test]
fn auto_turns_for_a_narrower_place_only() {
    let column = 674.0 * README_WIDTH / 880.0;
    let n = (2..12)
        .find(|&n| {
            let w = drawn(&chain(n, "right")).width;
            w > column && w <= README_WIDTH
        })
        .expect("a chain between the column's width and the README's");
    let shown =
        |json: String| json.replacen(r#""direction""#, r#""shownWidth": 674, "direction""#, 1);
    assert_eq!(drawn(&chain(n, "auto")).direction, Direction::Right);
    let narrow = drawn(&shown(chain(n, "auto")));
    assert_eq!(narrow.direction, Direction::Down);
    assert!(narrow.width <= column, "{}", narrow.width);
    // A named direction stays, however narrow the place.
    assert_eq!(drawn(&shown(chain(n, "right"))).direction, Direction::Right);
}
