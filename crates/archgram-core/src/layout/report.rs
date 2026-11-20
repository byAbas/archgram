//! How hard a drawing's lines are to follow, counted from the lines as laid
//! out (docs/features/approved-drawing.md): `build` prints the counts, and
//! the skill holds them at 0 where the user asked for nothing else. Text
//! only; the drawing is unchanged.

use std::collections::BTreeSet;

use crate::geometry::{Point, Rect};
use crate::spec::Spec;
use crate::tokens::SPACING_EDGE_EDGE;

use super::Placement;

/// Two arrivals this close or closer, side by side, read as one: two
/// tracks' spacing (`spacing.edge-edge`) and no more.
pub const SIDE_BY_SIDE_GAP: f64 = 2.0 * SPACING_EDGE_EDGE;

/// What makes a drawing's lines hard to follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
    /// Points where two lines cross.
    pub crossings: usize,
    /// Lines that pass through a frame they neither start nor end in, one
    /// for each such frame.
    pub through_frames: usize,
    /// Lines that run against the main direction: those that close a cycle.
    pub running_back: usize,
    /// Cards that lines from different sources reach side by side: their
    /// last pieces parallel, no more than [`SIDE_BY_SIDE_GAP`] apart, so
    /// the reader cannot tell which line comes from where.
    pub side_by_side: usize,
}

impl Counts {
    /// Whether nothing was counted.
    #[must_use]
    pub fn clear(&self) -> bool {
        *self == Counts::default()
    }
}

/// One straight piece of a line, its ends in order along its axis.
#[derive(Debug, Clone, Copy)]
struct Segment {
    horizontal: bool,
    /// The coordinate it keeps: y for a horizontal piece, x for a vertical.
    at: f64,
    from: f64,
    to: f64,
}

/// Half a pixel: what two drawn coordinates may differ by and still be one.
const SAME: f64 = 0.5;

fn segments(path: &[Point]) -> Vec<Segment> {
    path.windows(2)
        .filter_map(|w| {
            let (a, b) = (w[0], w[1]);
            if (a.y - b.y).abs() < SAME && (a.x - b.x).abs() >= SAME {
                Some(Segment {
                    horizontal: true,
                    at: a.y,
                    from: a.x.min(b.x),
                    to: a.x.max(b.x),
                })
            } else if (a.x - b.x).abs() < SAME && (a.y - b.y).abs() >= SAME {
                Some(Segment {
                    horizontal: false,
                    at: a.x,
                    from: a.y.min(b.y),
                    to: a.y.max(b.y),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Whether `v` lies strictly inside the span `from`..`to`.
fn inside(v: f64, from: f64, to: f64) -> bool {
    v > from + SAME && v < to - SAME
}

/// Counts what makes `placement`'s lines hard to follow.
#[must_use]
pub fn count(spec: &Spec, placement: &Placement) -> Counts {
    let lines: Vec<Vec<Segment>> = placement.edges.iter().map(|p| segments(p)).collect();
    Counts {
        crossings: crossings(&lines),
        through_frames: through_frames(spec, placement),
        running_back: super::acyclic_edges(spec).1.iter().filter(|&&r| r).count(),
        side_by_side: side_by_side(spec, &lines),
    }
}

/// Distinct points where a horizontal piece of one line crosses a vertical
/// piece of another, inside both: lines that only touch, at a bend or where
/// a bundle splits, do not cross.
fn crossings(lines: &[Vec<Segment>]) -> usize {
    let mut points = BTreeSet::new();
    for (i, a) in lines.iter().enumerate() {
        for b in &lines[i + 1..] {
            for s in a {
                for t in b {
                    let (h, v) = match (s.horizontal, t.horizontal) {
                        (true, false) => (s, t),
                        (false, true) => (t, s),
                        _ => continue,
                    };
                    if inside(v.at, h.from, h.to) && inside(h.at, v.from, v.to) {
                        // Whole tenths of a pixel, so one point counts once.
                        #[allow(clippy::cast_possible_truncation)]
                        let key = ((v.at * 10.0).round() as i64, (h.at * 10.0).round() as i64);
                        points.insert(key);
                    }
                }
            }
        }
    }
    points.len()
}

/// Each line that passes through a frame holding neither of its ends, once
/// for each such frame.
fn through_frames(spec: &Spec, placement: &Placement) -> usize {
    // The frames a node is in: its own and every frame around it.
    let around = |node: &str| -> BTreeSet<&str> {
        let mut out = BTreeSet::new();
        let mut at = spec
            .nodes
            .iter()
            .find(|n| n.id == node)
            .and_then(|n| n.frame.as_deref());
        while let Some(f) = at {
            if !out.insert(f) {
                break;
            }
            at = spec
                .frames
                .iter()
                .find(|fr| fr.id == f)
                .and_then(|fr| fr.parent.as_deref());
        }
        out
    };
    let mut count = 0;
    for (edge, path) in spec.edges.iter().zip(&placement.edges) {
        let ends: BTreeSet<&str> = around(&edge.from)
            .union(&around(&edge.to))
            .copied()
            .collect();
        for (frame, rect) in spec.frames.iter().zip(&placement.frames) {
            let Some(rect) = rect else { continue };
            if ends.contains(frame.id.as_str()) {
                continue;
            }
            if segments(path).iter().any(|s| enters(s, *rect)) {
                count += 1;
            }
        }
    }
    count
}

/// Whether a piece of line passes inside a frame's box.
fn enters(s: &Segment, r: Rect) -> bool {
    let (across, along_from, along_to) = if s.horizontal {
        (inside(s.at, r.y, r.bottom()), r.x, r.right())
    } else {
        (inside(s.at, r.x, r.right()), r.y, r.bottom())
    };
    across && s.to > along_from + SAME && s.from < along_to - SAME
}

/// Cards reached by lines from different sources whose arrivals run side
/// by side.
fn side_by_side(spec: &Spec, lines: &[Vec<Segment>]) -> usize {
    let mut cards = BTreeSet::new();
    for (i, a) in spec.edges.iter().enumerate() {
        for (j, b) in spec.edges.iter().enumerate().skip(i + 1) {
            if a.to != b.to || a.from == b.from {
                continue;
            }
            let together = match (lines[i].last(), lines[j].last()) {
                (Some(s), Some(t)) => {
                    s.horizontal == t.horizontal
                        && (s.at - t.at).abs() <= SIDE_BY_SIDE_GAP
                        && s.to.min(t.to) - s.from.max(t.from) > SAME
                }
                _ => false,
            };
            if together {
                cards.insert(a.to.as_str());
            }
        }
    }
    cards.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    fn laid_out(json: &str) -> (Spec, Placement) {
        let spec = crate::parse_spec(json).expect("a valid spec");
        let placement =
            super::super::place(&spec, &crate::measure::card_sizes(&spec)).expect("laid out");
        (spec, placement)
    }

    fn line(points: &[(f64, f64)]) -> Vec<Point> {
        points.iter().map(|&(x, y)| Point { x, y }).collect()
    }

    const NODES: &str = r#"[
        { "id": "a", "kind": "service", "label": "A" },
        { "id": "b", "kind": "service", "label": "B", "frame": "f" },
        { "id": "c", "kind": "service", "label": "C" },
        { "id": "d", "kind": "service", "label": "D" }
    ]"#;

    fn spec(edges: &str) -> String {
        format!(
            r#"{{ "archgram": 1, "title": "t", "description": "d", "nodes": {NODES},
                 "frames": [{{ "id": "f", "label": "F" }}], "edges": {edges} }}"#
        )
    }

    /// One line from a card to each of three others, none back: nothing to count.
    #[test]
    fn a_fan_out_counts_nothing() {
        let (s, p) = laid_out(&spec(
            r#"[{ "from": "a", "to": "b" }, { "from": "a", "to": "c" }, { "from": "a", "to": "d" }]"#,
        ));
        assert!(count(&s, &p).clear(), "{:?}", count(&s, &p));
    }

    /// A line that closes a cycle runs back.
    #[test]
    fn a_cycle_runs_back() {
        let (s, p) = laid_out(&spec(
            r#"[{ "from": "a", "to": "c" }, { "from": "c", "to": "a" }]"#,
        ));
        assert_eq!(count(&s, &p).running_back, 1);
    }

    /// Two lines from different cards into one, their arrivals parallel and
    /// a track apart, reach it side by side; lines from one card do not.
    #[test]
    fn arrivals_from_two_sources_side_by_side() {
        let (s, mut p) = laid_out(&spec(
            r#"[{ "from": "a", "to": "d" }, { "from": "c", "to": "d" }]"#,
        ));
        p.edges = vec![
            line(&[(0.0, 100.0), (300.0, 100.0)]),
            line(&[(200.0, 40.0), (200.0, 112.0), (300.0, 112.0)]),
        ];
        assert_eq!(count(&s, &p).side_by_side, 1);
        p.edges[1] = line(&[(200.0, 40.0), (200.0, 200.0), (300.0, 200.0)]);
        assert_eq!(count(&s, &p).side_by_side, 0, "far apart");
    }

    /// A horizontal piece through a vertical one of another line is one
    /// crossing; touching at an end is none.
    #[test]
    fn crossings_are_inside_both_pieces() {
        let (s, mut p) = laid_out(&spec(
            r#"[{ "from": "a", "to": "d" }, { "from": "c", "to": "b" }]"#,
        ));
        p.edges = vec![
            line(&[(0.0, 100.0), (300.0, 100.0)]),
            line(&[(150.0, 0.0), (150.0, 200.0)]),
        ];
        assert_eq!(count(&s, &p).crossings, 1);
        p.edges[1] = line(&[(150.0, 100.0), (150.0, 200.0)]);
        assert_eq!(count(&s, &p).crossings, 0, "a T is no crossing");
    }

    /// A line through a frame that holds neither of its ends is counted;
    /// one into the frame is not.
    #[test]
    fn through_a_frame_holding_neither_end() {
        let (s, mut p) = laid_out(&spec(
            r#"[{ "from": "a", "to": "c" }, { "from": "a", "to": "b" }]"#,
        ));
        let f = p.frames[0].expect("the frame is placed");
        let y = f.y + f.h / 2.0;
        p.edges = vec![
            line(&[(f.x - 50.0, y), (f.right() + 50.0, y)]),
            line(&[(f.x - 50.0, y), (f.x + 10.0, y)]),
        ];
        assert_eq!(count(&s, &p).through_frames, 1);
    }
}
