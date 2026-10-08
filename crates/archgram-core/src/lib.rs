//! archgram's engine: a spec in, a picture out, with no I/O
//! (ARCHITECTURE.md, Bird's eye view).

pub mod color;
mod error;
pub mod font;
pub mod geometry;
pub mod layout;
pub mod logos;
mod math;
pub mod measure;
pub mod motion;
pub mod render;
pub mod sources;
pub mod spec;
pub mod theme;
pub mod tokens;
mod validate;

pub use error::{Location, SpecError};
pub use spec::Spec;
pub use validate::{FORMAT_VERSION, validate};

use spec::Direction;

/// The widest drawing a README on GitHub shows with its text at a readable
/// size (docs/PRD.md 6.6): `direction: auto` keeps left to right within
/// it, and otherwise chooses the narrower direction.
pub const README_WIDTH: f64 = 1300.0;

/// How wide a README on GitHub shows a drawing, about: the width
/// [`README_WIDTH`] is measured against.
pub const README_SHOWN_WIDTH: f64 = 880.0;

/// The widest drawing that keeps its text readable where `spec` is shown
/// (docs/features/shown-width.md): [`README_WIDTH`] in a README, and
/// `shownWidth` scaled by the same ratio elsewhere, so the smallest text
/// shows at the size it does in a README.
#[must_use]
pub fn readable_width(spec: &Spec) -> f64 {
    spec.shown_width.map_or(README_WIDTH, |shown| {
        shown * README_WIDTH / README_SHOWN_WIDTH
    })
}

/// A drawing, with its size and the direction it was laid out in.
#[derive(Debug, Clone, PartialEq)]
pub struct Drawing {
    pub svg: String,
    pub width: f64,
    pub height: f64,
    /// Right or down: the spec's own direction, or what `auto` came to.
    pub direction: Direction,
}

/// Reads a JSON spec and checks it. Returns the spec, or every problem found:
/// one problem when the text is not a well-formed spec, all of them when it
/// is well-formed but breaks the rules in docs/SPEC.md.
///
/// # Errors
///
/// The problems, each with its location in the spec.
pub fn parse_spec(json: &str) -> Result<Spec, Vec<SpecError>> {
    let spec: Spec = serde_json::from_str(json).map_err(|e| {
        vec![SpecError {
            location: Location::LineColumn {
                line: e.line(),
                column: e.column(),
            },
            message: without_quoted_text(
                e.to_string().split(" at line ").next().unwrap_or_default(),
            ),
        }]
    })?;
    let errors = validate(&spec);
    if errors.is_empty() {
        Ok(spec)
    } else {
        Err(errors)
    }
}

/// serde's message without a text it quotes from the input: `invalid type:
/// string "…"` names the kind of value, not the value, so a file read as a
/// spec by mistake is never printed back (SECURITY.md).
fn without_quoted_text(message: &str) -> String {
    const LEAD: &str = "invalid type: string \"";
    if let (Some(start), Some(end)) = (message.find(LEAD), message.rfind("\", expected"))
        && end + 1 >= start + LEAD.len()
    {
        return format!(
            "{}invalid type: text{}",
            &message[..start],
            &message[end + 1..]
        );
    }
    message.to_owned()
}

/// Reads, checks and draws a JSON spec: the whole pipeline.
///
/// # Errors
///
/// The spec's problems, as [`parse_spec`] reports them.
pub fn build(json: &str, options: render::Options) -> Result<String, Vec<SpecError>> {
    build_with(json, options, &logos::NoLogos)
}

/// [`build`], with technology logos drawn from `logos`; each `tech` must
/// name one of them.
///
/// # Errors
///
/// The spec's problems, as [`parse_spec`] reports them, and each `tech`
/// that `logos` does not have.
pub fn build_with(
    json: &str,
    options: render::Options,
    logos: &dyn logos::Logos,
) -> Result<String, Vec<SpecError>> {
    let spec = parse_spec(json)?;
    let errors = validate::logos(&spec, logos);
    if !errors.is_empty() {
        return Err(errors);
    }
    draw_with(&spec, options, logos).map(|drawing| drawing.svg)
}

/// Measures, lays out and draws a spec that has passed validation.
///
/// # Errors
///
/// Layout hints that contradict the edges.
pub fn draw(spec: &Spec, options: render::Options) -> Result<String, Vec<SpecError>> {
    draw_with(spec, options, &logos::NoLogos).map(|drawing| drawing.svg)
}

/// [`draw`], with technology logos drawn from `logos` where a node names
/// one it has, and with the drawing's size and direction.
///
/// # Errors
///
/// Layout hints that contradict the edges.
pub fn draw_with(
    spec: &Spec,
    options: render::Options,
    logos: &dyn logos::Logos,
) -> Result<Drawing, Vec<SpecError>> {
    let (spec, placement) = fit(spec, options, logos)?;
    let scene = render::scene(&spec, &placement, options, logos);
    Ok(Drawing {
        svg: render::svg::write(&scene),
        width: scene.width,
        height: scene.height,
        direction: spec.direction,
    })
}

/// The spec laid out in its direction, `auto` decided (docs/SPEC.md, Top
/// level): left to right when the drawing is no wider than
/// [`readable_width`], and otherwise the narrower of the two directions, so
/// a wide, shallow diagram does not turn wider still. The spec returned
/// names the direction it was laid out in, so layout only ever sees right
/// or down.
fn fit(
    spec: &Spec,
    options: render::Options,
    logos: &dyn logos::Logos,
) -> Result<(Spec, layout::Placement), Vec<SpecError>> {
    let sizes = measure::card_sizes_with(spec, logos);
    let lay = |direction| {
        let spec = Spec {
            direction,
            ..spec.clone()
        };
        let placement = layout::place(&spec, &sizes)?;
        Ok((spec, placement))
    };
    if spec.direction != Direction::Auto {
        return lay(spec.direction);
    }
    let width = |(spec, placement): &(Spec, layout::Placement)| {
        render::scene(spec, placement, options, logos).width
    };
    let right = lay(Direction::Right)?;
    if width(&right) <= readable_width(spec) {
        return Ok(right);
    }
    let down = lay(Direction::Down)?;
    Ok(if width(&down) < width(&right) {
        down
    } else {
        right
    })
}

/// The spec laid out once and drawn twice, light then dark, each file with
/// one theme only: for a page that chooses one per reader, such as GitHub's
/// `<picture>` with `prefers-color-scheme` sources.
///
/// # Errors
///
/// Layout hints that contradict the edges.
pub fn draw_themes(
    spec: &Spec,
    options: render::Options,
    logos: &dyn logos::Logos,
) -> Result<(Drawing, Drawing), Vec<SpecError>> {
    let (spec, placement) = fit(spec, options, logos)?;
    let theme = |mode| {
        let scene = render::scene(
            &spec,
            &placement,
            render::Options { mode, ..options },
            logos,
        );
        Drawing {
            svg: render::svg::write(&scene),
            width: scene.width,
            height: scene.height,
            direction: spec.direction,
        }
    };
    Ok((theme(render::Mode::Light), theme(render::Mode::Dark)))
}

/// Each `tech` that `logos` does not have, located in the spec, with the
/// nearest slugs suggested. Empty when `logos` has none at all.
#[must_use]
pub fn check_logos(spec: &Spec, logos: &dyn logos::Logos) -> Vec<SpecError> {
    validate::logos(spec, logos)
}

/// The characters in the spec's text that the embedded font does not have,
/// in order and without repeats. They are drawn in the reader's font.
#[must_use]
pub fn uncovered_characters(spec: &Spec, logos: &dyn logos::Logos) -> Vec<char> {
    let mut out: Vec<char> = Vec::new();
    for (weight, text) in measure::text_runs_with(spec, logos) {
        for c in font::uncovered(&text, weight) {
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}
