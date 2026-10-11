//! The style table: every class a still drawing's shapes name, with its
//! declarations (DESIGN.md, front matter: components). The SVG writes it as
//! CSS; a rasterizer reads the same rules, so a PNG looks as the SVG does.

use crate::render::scene::{Decl, Paint, Rule};
use crate::tokens::{
    DASH_EDGE, DASH_EXTERNAL, DASH_FRAGMENT, DASH_FRAME, DASH_LIFELINE, DASH_OPERAND, DASH_REPLY,
    GRID_OPACITY, Role, STROKE_CARD, STROKE_CONNECTOR, STROKE_FRAME, STROKE_MESSAGE,
    TYPOGRAPHY_FRAGMENT, TYPOGRAPHY_FRAME_LABEL, TYPOGRAPHY_LEGEND, TYPOGRAPHY_MESSAGE,
    TYPOGRAPHY_MESSAGE_NUMBER, TYPOGRAPHY_MESSAGE_REPLY, TYPOGRAPHY_PHASE, TYPOGRAPHY_SUBTITLE,
    TYPOGRAPHY_TITLE,
};

/// The four category hues, by class name, with their roles.
pub const HUES: [(&str, Role); 4] = [
    ("core", Role::IconCore),
    ("ai", Role::IconAi),
    ("build", Role::IconBuild),
    ("client", Role::IconClient),
];

fn rule(selector: impl Into<String>, decls: &[Decl]) -> Rule {
    Rule {
        selector: selector.into(),
        decls: decls.to_vec(),
    }
}

use Decl::{Dash, Fill, Font, Opacity, RoundCaps, RoundJoins, Stroke, StrokeWidth};
use Paint::{None as NoPaint, Role as R};

/// The rules every drawing has: cards, icons, logos, frames and edges.
#[must_use]
pub fn shapes() -> Vec<Rule> {
    let mut out = vec![
        rule(".title", &[Font(TYPOGRAPHY_TITLE), Fill(R(Role::Text))]),
        rule(
            ".sub",
            &[Font(TYPOGRAPHY_SUBTITLE), Fill(R(Role::TextMuted))],
        ),
        rule(".canvas", &[Fill(R(Role::Canvas))]),
        rule(
            ".card",
            &[
                Fill(R(Role::Card)),
                Stroke(R(Role::CardEdge)),
                StrokeWidth(STROKE_CARD),
            ],
        ),
        rule(
            ".card.external",
            &[
                Fill(R(Role::Canvas)),
                Stroke(R(Role::Connector)),
                Dash(DASH_EXTERNAL),
            ],
        ),
        rule(".badge", &[Fill(R(Role::Badge))]),
        rule(".logo", &[Fill(R(Role::TextMuted))]),
    ];
    out.extend(
        HUES.iter()
            .map(|&(hue, role)| rule(format!(".logo-icon.{hue}"), &[Fill(R(role))])),
    );
    out.extend([
        rule(
            ".logo-chip",
            &[
                Fill(R(Role::Card)),
                Stroke(R(Role::CardEdge)),
                StrokeWidth(STROKE_CARD),
            ],
        ),
        rule(
            ".frame",
            &[
                Fill(NoPaint),
                Stroke(R(Role::Frame)),
                StrokeWidth(STROKE_FRAME),
                Dash(DASH_FRAME),
            ],
        ),
        rule(
            ".frame-label",
            &[Font(TYPOGRAPHY_FRAME_LABEL), Fill(R(Role::TextMuted))],
        ),
        rule(".label-patch", &[Fill(R(Role::Canvas))]),
        rule(
            ".edge",
            &[
                Fill(NoPaint),
                Stroke(R(Role::Connector)),
                StrokeWidth(STROKE_CONNECTOR),
                RoundCaps,
                RoundJoins,
            ],
        ),
        rule(".edge.dashed", &[Dash(DASH_EDGE)]),
        rule(
            ".arrowhead",
            &[
                Fill(NoPaint),
                Stroke(R(Role::Connector)),
                StrokeWidth(STROKE_CONNECTOR),
                RoundCaps,
                RoundJoins,
            ],
        ),
        rule(".icon", &[Fill(NoPaint), RoundCaps, RoundJoins]),
    ]);
    out.extend(
        HUES.iter()
            .map(|&(hue, role)| rule(format!(".icon.{hue}"), &[Stroke(R(role))])),
    );
    out
}

/// A step's number on a line, in the still image that numbers the flows: a
/// badge edged like a line, its number like text.
#[must_use]
pub fn steps() -> Vec<Rule> {
    vec![
        rule(
            ".step",
            &[
                Fill(R(Role::Card)),
                Stroke(R(Role::Connector)),
                StrokeWidth(STROKE_CARD),
            ],
        ),
        rule(
            ".step-text",
            &[Font(TYPOGRAPHY_LEGEND), Fill(R(Role::Text))],
        ),
    ]
}

/// How much of the credit shows: it signs the drawing, and says nothing a
/// reader needs.
const CREDIT_OPACITY: f64 = 0.6;

/// The credit, quieter than any text: the legend's type and the mark in
/// the connector's colour, the mark's lines cut out of it in the canvas's
/// (DESIGN.md, Components: Credit).
#[must_use]
pub fn credit() -> Vec<Rule> {
    vec![
        // Faded as one, so the mark's cut lines stay cut.
        rule(".credit-line", &[Opacity(CREDIT_OPACITY)]),
        rule(
            ".credit",
            &[Font(TYPOGRAPHY_LEGEND), Fill(R(Role::Connector))],
        ),
        rule(".credit-mark", &[Fill(R(Role::Connector))]),
        rule(
            ".credit-glyph",
            &[
                Fill(NoPaint),
                Stroke(R(Role::Canvas)),
                // The logo's own stroke, in its 64 grid (docs/images/logo.svg).
                StrokeWidth(5.5),
                RoundCaps,
                RoundJoins,
            ],
        ),
    ]
}

/// The legend's swatches and text.
#[must_use]
pub fn legend() -> Vec<Rule> {
    vec![rule(
        ".legend-text",
        &[Font(TYPOGRAPHY_LEGEND), Fill(R(Role::TextMuted))],
    )]
}

/// A sequence's own parts (DESIGN.md, Components: Sequence), drawn as a
/// blueprint: its grid, phases and lifelines; each kind of message's line,
/// arrowhead, number and travelling point in its colour; the bars while a
/// call is answered; a fragment's frame, pills and operand lines; each
/// label's patch and words; and the key.
#[must_use]
#[allow(clippy::too_many_lines)] // one rule per part, in drawing order
pub fn sequence() -> Vec<Rule> {
    let kinds = [
        ("call", Role::MessageCall),
        ("send", Role::MessageSend),
        ("reply", Role::Connector),
    ];
    let mut rules = vec![
        rule(".grid-dot", &[Fill(R(Role::Frame)), Opacity(GRID_OPACITY)]),
        rule(".phase", &[Fill(R(Role::Card))]),
        rule(
            ".phase-label",
            &[Font(TYPOGRAPHY_PHASE), Fill(R(Role::TextMuted))],
        ),
        rule(".phase-label.lit", &[Fill(R(Role::Text))]),
        rule(".label-patch.band", &[Fill(R(Role::Card))]),
        rule(
            ".lifeline",
            &[
                Fill(NoPaint),
                Stroke(R(Role::Frame)),
                StrokeWidth(STROKE_CARD),
                Dash(DASH_LIFELINE),
            ],
        ),
    ];
    for (kind, role) in kinds {
        rules.push(rule(
            format!(".edge.{kind}"),
            &[Stroke(R(role)), StrokeWidth(STROKE_MESSAGE)],
        ));
        rules.push(rule(
            format!(".arrowhead.{kind}"),
            &[Stroke(R(role)), StrokeWidth(STROKE_MESSAGE)],
        ));
    }
    rules.extend([
        rule(".edge.reply", &[Dash(DASH_REPLY)]),
        rule(".arrowhead.filled", &[Fill(R(Role::MessageCall))]),
        rule(
            ".activation",
            &[
                Fill(R(Role::Card)),
                Stroke(R(Role::CardEdge)),
                StrokeWidth(STROKE_CARD),
            ],
        ),
        rule(
            ".fragment",
            &[
                Fill(NoPaint),
                Stroke(R(Role::Frame)),
                StrokeWidth(STROKE_FRAME),
            ],
        ),
        rule(".fragment.dashed", &[Dash(DASH_FRAGMENT)]),
        rule(
            ".fragment-tag",
            &[
                Fill(R(Role::Badge)),
                Stroke(R(Role::Frame)),
                StrokeWidth(STROKE_FRAME),
            ],
        ),
        rule(
            ".fragment-op",
            &[Font(TYPOGRAPHY_FRAGMENT), Fill(R(Role::Text))],
        ),
        rule(
            ".operand",
            &[
                Fill(NoPaint),
                Stroke(R(Role::Frame)),
                StrokeWidth(STROKE_FRAME),
                Dash(DASH_OPERAND),
            ],
        ),
        rule(
            ".message-number",
            &[Font(TYPOGRAPHY_MESSAGE_NUMBER), Fill(R(Role::MessageCall))],
        ),
        rule(".message-number.send", &[Fill(R(Role::MessageSend))]),
        rule(".message-number.reply", &[Fill(R(Role::TextMuted))]),
        rule(
            ".message-text",
            &[Font(TYPOGRAPHY_MESSAGE), Fill(R(Role::Text))],
        ),
        rule(
            ".message-text.reply",
            &[Font(TYPOGRAPHY_MESSAGE_REPLY), Fill(R(Role::TextMuted))],
        ),
        rule(".key", &[Fill(NoPaint), StrokeWidth(STROKE_MESSAGE)]),
    ]);
    for (kind, role) in kinds {
        rules.push(rule(format!(".key.{kind}"), &[Stroke(R(role))]));
    }
    rules.extend([
        rule(".key.reply", &[Dash(DASH_REPLY)]),
        rule(".key.refused", &[Stroke(R(Role::SignalRefusal))]),
    ]);
    rules
}

/// A refused message in a sequence: its ✕ on a patch of what it sits on,
/// its line, arrowhead, number and words in the refusal colour, and the
/// refusing head's border while its refusal plays (DESIGN.md, Components:
/// Refusal).
#[must_use]
pub fn sequence_refusal() -> Vec<Rule> {
    vec![
        rule(
            ".refused-mark",
            &[
                Fill(NoPaint),
                Stroke(R(Role::SignalRefusal)),
                StrokeWidth(STROKE_CONNECTOR),
                RoundCaps,
                RoundJoins,
            ],
        ),
        rule(
            ".refused-patch",
            &[
                Fill(NoPaint),
                Stroke(R(Role::Canvas)),
                StrokeWidth(3.0 * STROKE_CONNECTOR),
            ],
        ),
        rule(".refused-patch.band", &[Stroke(R(Role::Card))]),
        rule(".edge.refused", &[Stroke(R(Role::SignalRefusal))]),
        rule(".arrowhead.refused", &[Stroke(R(Role::SignalRefusal))]),
        rule(".message-number.refused", &[Fill(R(Role::SignalRefusal))]),
        rule(".message-text.refused", &[Fill(R(Role::SignalRefusal))]),
        rule(
            ".refused-head",
            &[
                Fill(NoPaint),
                Stroke(R(Role::SignalRefusal)),
                StrokeWidth(STROKE_CARD),
            ],
        ),
    ]
}
