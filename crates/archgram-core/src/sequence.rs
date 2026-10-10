//! Sequence diagrams: the messages between participants, in order
//! (docs/features/sequence.md; docs/SPEC.md, Sequence). Their meaning is
//! UML's (Unified Modeling Language 2.5.1, clause 17); archgram reads the
//! part of it the feature names.
//!
//! A message, a reply and a fragment are one type, [`Item`], with every
//! field optional, and [`Item::what`] says which it is. serde reads it with
//! unknown fields refused, so a misspelt field is located at its line; an
//! untagged enum would say only that no variant matched.

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::error::SpecError;
use crate::sources::Owned;
use crate::spec::{
    BorderStyle, CardStyle, DiagramKind, Kind, LogoPlace, SignalStyle, Sources, Variant, Wait,
    default_palette, default_true,
};
use crate::validate::{
    Common, is_valid_id, nearest, source_forms, tech_form, techs_carried, top_level, xml_problem,
};

/// A whole sequence diagram.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    /// The spec format's version. Only `1` exists.
    pub archgram: u32,
    /// `sequence`, which made this spec read as one.
    pub diagram: DiagramKind,
    /// The diagram's name, written as the SVG's `<title>`.
    pub title: String,
    /// The whole diagram in prose, written as the SVG's `<desc>`.
    pub description: String,
    /// How wide the diagram is shown, in CSS pixels; a README on GitHub
    /// when absent.
    #[serde(default, rename = "shownWidth")]
    pub shown_width: Option<f64>,
    #[serde(default)]
    pub card: CardStyle,
    #[serde(default)]
    pub logo: LogoPlace,
    #[serde(default = "default_palette")]
    pub palette: String,
    #[serde(default)]
    pub signal: SignalStyle,
    /// What the still image shows: each message's number, by default.
    #[serde(default)]
    pub still: SequenceStill,
    /// How it is drawn: cards and activation bars, by default.
    #[serde(default)]
    pub look: Look,
    #[serde(default)]
    pub border: BorderStyle,
    #[serde(default)]
    pub wait: Wait,
    #[serde(default = "default_true")]
    pub glow: bool,
    #[serde(default = "default_true")]
    pub credit: bool,
    /// Left to right, in this order, each over its lifeline.
    pub participants: Vec<Participant>,
    /// Time's order, down the page.
    pub messages: Vec<Item>,
}

/// What a sequence's still image shows, where nothing moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SequenceStill {
    /// Each message's number by its line: in a sequence the order is the
    /// meaning.
    #[default]
    Numbers,
    /// Nothing more than the diagram.
    None,
}

/// How a sequence is drawn (docs/features/sequence.md, Looks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Look {
    /// Each head a card, and a bar on a lifeline while its participant
    /// answers a call.
    #[default]
    Cards,
    /// Each head a circle, and no bars.
    Avatars,
}

/// One participant: a node's fields, without a frame.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Participant {
    pub id: String,
    pub kind: Kind,
    pub label: String,
    #[serde(default)]
    pub note: Option<String>,
    /// A Simple Icons slug naming the participant's technology.
    #[serde(default)]
    pub tech: Option<String>,
    #[serde(default)]
    pub variant: Variant,
    /// What backs the participant, its code or its document. Never drawn.
    #[serde(default)]
    pub source: Option<Sources>,
}

/// One item of a `messages` list, read as written: a message (`from`,
/// `to`), a reply (`reply`), or a fragment (`alt`, `opt`, `loop`, `par`).
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    /// The participant that replies.
    #[serde(default)]
    pub reply: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    /// A send that does not wait for a reply.
    #[serde(default, rename = "async")]
    pub sends: bool,
    /// The message turns the request away (archgram's, not UML's).
    #[serde(default)]
    pub refused: bool,
    /// The code that sends the message. Never drawn.
    #[serde(default)]
    pub source: Option<Sources>,
    #[serde(default)]
    pub alt: Option<Vec<Operand>>,
    #[serde(default)]
    pub opt: Option<Operand>,
    #[serde(default, rename = "loop")]
    pub repeat: Option<Operand>,
    #[serde(default)]
    pub par: Option<Vec<Operand>>,
}

/// One operand of a fragment: its guard, and its own messages.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operand {
    /// The guard, drawn in its fragment's pill; `else` on an `alt`'s last.
    #[serde(default)]
    pub when: Option<String>,
    /// The code that makes the choice, such as the `if` of the guard.
    /// Never drawn.
    #[serde(default)]
    pub source: Option<Sources>,
    pub messages: Vec<Item>,
}

/// A fragment's operator (UML 2.5.1, 17.12.15, `InteractionOperatorKind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// One of several operands, the first whose guard holds.
    Alt,
    /// The operand happens, or nothing does.
    Opt,
    /// The operand is repeated.
    Loop,
    /// The operands interleave, each in its own order.
    Par,
}

impl Operator {
    /// The operator as the spec writes it and the drawing shows it.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Operator::Alt => "alt",
            Operator::Opt => "opt",
            Operator::Loop => "loop",
            Operator::Par => "par",
        }
    }
}

/// What an [`Item`] is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum What<'a> {
    /// A call, or a send that does not wait.
    Message { from: &'a str, to: &'a str },
    /// A reply from `by`, to `to` when the spec names it.
    Reply { by: &'a str, to: Option<&'a str> },
    /// A fragment and its operands.
    Fragment {
        operator: Operator,
        operands: &'a [Operand],
    },
}

impl Item {
    /// What the item is, or why it is none: the fields of exactly one
    /// form, a message, a reply or a fragment.
    ///
    /// # Errors
    ///
    /// Why the item is not one of the three.
    pub fn what(&self) -> Result<What<'_>, String> {
        let fragments: Vec<(Operator, &[Operand])> = [
            (Operator::Alt, self.alt.as_deref()),
            (Operator::Opt, self.opt.as_ref().map(std::slice::from_ref)),
            (
                Operator::Loop,
                self.repeat.as_ref().map(std::slice::from_ref),
            ),
            (Operator::Par, self.par.as_deref()),
        ]
        .into_iter()
        .filter_map(|(op, operands)| operands.map(|o| (op, o)))
        .collect();
        let message = self.from.is_some() || self.to.is_some();
        let reply = self.reply.is_some();
        let forms = usize::from(message && !reply) + usize::from(reply) + fragments.len();
        if forms == 0 {
            return Err(
                "an item is a message (`from`, `to`), a reply (`reply`), or a fragment (`alt`, `opt`, `loop`, `par`)"
                    .into(),
            );
        }
        if let [(operator, operands)] = fragments.as_slice() {
            let name = operator.name();
            if forms > 1 {
                return Err(format!(
                    "a fragment is an item of its own: put `{name}` in an item without `from`, `to` or `reply`"
                ));
            }
            if self.label.is_some() || self.sends || self.refused || self.source.is_some() {
                return Err(format!(
                    "a fragment holds operands only; `label`, `async`, `refused` and `source` belong to its messages, not to `{name}`"
                ));
            }
            return Ok(What::Fragment {
                operator: *operator,
                operands,
            });
        }
        if !fragments.is_empty() {
            return Err("an item holds one fragment; nest one inside another's operand".into());
        }
        if let Some(by) = &self.reply {
            if self.from.is_some() {
                return Err(format!(
                    "a reply names who replies in `reply`, not in `from`: write `reply: {by}`"
                ));
            }
            if self.sends {
                return Err("a reply does not wait for anything; leave `async` out".into());
            }
            return Ok(What::Reply {
                by,
                to: self.to.as_deref(),
            });
        }
        match (&self.from, &self.to) {
            (Some(from), Some(to)) => Ok(What::Message { from, to }),
            (Some(_), None) => Err("a message needs `to`, the participant it goes to".into()),
            _ => Err("a message needs `from`, the participant that sends it; a reply names who replies in `reply`".into()),
        }
    }
}

impl Sequence {
    /// The participants as an architecture spec's nodes, with the
    /// sequence's look: so a head is measured, drawn and listed in the
    /// legend as a card is, and its text set in the same font.
    #[must_use]
    pub fn as_spec(&self) -> crate::Spec {
        crate::Spec {
            archgram: self.archgram,
            diagram: DiagramKind::Architecture,
            title: self.title.clone(),
            description: self.description.clone(),
            direction: crate::spec::Direction::Right,
            shown_width: self.shown_width,
            card: self.card,
            logo: self.logo,
            palette: self.palette.clone(),
            legend: true,
            signal: self.signal,
            still: crate::spec::Still::None,
            border: self.border,
            wait: self.wait,
            glow: self.glow,
            credit: self.credit,
            nodes: self
                .participants
                .iter()
                .map(|p| crate::spec::Node {
                    id: p.id.clone(),
                    kind: p.kind,
                    label: p.label.clone(),
                    note: p.note.clone(),
                    tech: p.tech.clone(),
                    variant: p.variant,
                    frame: None,
                    source: None,
                })
                .collect(),
            frames: Vec::new(),
            edges: Vec::new(),
            flows: Vec::new(),
            hints: crate::spec::Hints::default(),
        }
    }

    /// Every participant and message with its sources, for `check`.
    #[must_use]
    pub fn owned(&self) -> Vec<Owned<'_>> {
        let mut owned: Vec<Owned<'_>> = self
            .participants
            .iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    format!("/participants/{i}/source"),
                    format!("participant {}", p.id),
                    &p.source,
                )
            })
            .collect();
        walk(
            &self.messages,
            "/messages",
            &mut |pointer, item| match item.what() {
                Ok(What::Message { from, to }) => owned.push((
                    format!("{pointer}/source"),
                    format!("message {from} \u{2192} {to}"),
                    &item.source,
                )),
                Ok(What::Reply { by, .. }) => owned.push((
                    format!("{pointer}/source"),
                    format!("reply from {by}"),
                    &item.source,
                )),
                Ok(What::Fragment { operator, operands }) => {
                    for (j, operand) in operands.iter().enumerate() {
                        owned.push((
                            format!("{}/source", operand_at(pointer, operator, j)),
                            format!("guard of {}", operator.name()),
                            &operand.source,
                        ));
                    }
                }
                Err(_) => {}
            },
        );
        owned
    }

    /// How many messages and replies the sequence holds, in every operand.
    #[must_use]
    pub fn message_count(&self) -> usize {
        let mut n = 0;
        walk(&self.messages, "/messages", &mut |_, item| {
            if matches!(item.what(), Ok(What::Message { .. } | What::Reply { .. })) {
                n += 1;
            }
        });
        n
    }
}

/// How a message is drawn (UML 2.5.1, 17.4.4.1): a call waits for its
/// reply, a send does not, a reply answers a call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Call,
    Send,
    Reply,
}

/// A message in time's order, its participants by their place in the
/// list, a reply's receiver resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolved<'a> {
    pub from: usize,
    pub to: usize,
    pub sort: Sort,
    pub refused: bool,
    pub label: Option<&'a str>,
    /// For a reply, the call it answers, by its place among the messages
    /// in time's order.
    pub answers: Option<usize>,
}

/// One step of a sequence read in time's order.
#[derive(Debug, Clone, PartialEq)]
pub enum Step<'a> {
    Message(Resolved<'a>),
    /// A fragment opens, with its first operand's guard.
    Open(Operator, Option<&'a str>),
    /// Its next operand starts, with its guard.
    Operand(Option<&'a str>),
    Close,
}

/// The latest call to `by` still waiting, which a reply from `by`
/// answers: its caller, and what the reader keeps of it. That call stops
/// waiting. The one rule of replies, for validation, layout and the words
/// a screen reader hears.
pub(crate) fn answer<T: PartialEq + Copy, C>(open: &mut Vec<(T, T, C)>, by: &T) -> Option<(T, C)> {
    open.iter()
        .rposition(|(_, callee, _)| callee == by)
        .map(|k| {
            let (caller, _, kept) = open.remove(k);
            (caller, kept)
        })
}

/// Reads a fragment's `count` operands with `operand`, each from the calls
/// waiting before it, and leaves in `open` the calls waiting after it: for
/// `par` its operands one after another; for `alt`, `opt` and `loop` every
/// call left waiting by any way through (for `opt` and `loop`, by none
/// too), those waiting before it first, so a later reply is not refused for
/// one way.
pub(crate) fn through<T: PartialEq + Clone>(
    operator: Operator,
    count: usize,
    open: &mut Vec<T>,
    operand: &mut dyn FnMut(usize, &mut Vec<T>),
) {
    if operator == Operator::Par {
        for j in 0..count {
            operand(j, open);
        }
        return;
    }
    let before = open.clone();
    let mut after: Vec<T> = match operator {
        Operator::Opt | Operator::Loop => before.clone(),
        Operator::Alt | Operator::Par => Vec::new(),
    };
    for j in 0..count {
        let mut way = before.clone();
        operand(j, &mut way);
        for call in way {
            if !after.contains(&call) {
                after.push(call);
            }
        }
    }
    let mut kept: Vec<T> = before
        .iter()
        .filter(|c| after.contains(c))
        .cloned()
        .collect();
    kept.extend(after.into_iter().filter(|c| !before.contains(c)));
    *open = kept;
}

impl Sequence {
    /// The sequence in time's order: each message with its participants by
    /// place and each reply's receiver resolved, and where each fragment
    /// and operand opens and closes. What the layout draws and a screen
    /// reader hears. The sequence must have passed validation.
    ///
    /// # Panics
    ///
    /// When a message names a participant that does not exist.
    #[must_use]
    pub fn steps(&self) -> Vec<Step<'_>> {
        let mut out = Vec::new();
        self.read(&self.messages, &mut Vec::new(), &mut out);
        out
    }

    fn index(&self, id: &str) -> usize {
        self.participants
            .iter()
            .position(|p| p.id == id)
            .expect("validated: every message names a participant")
    }

    fn read<'a>(
        &'a self,
        items: &'a [Item],
        open: &mut Vec<(usize, usize, usize)>,
        out: &mut Vec<Step<'a>>,
    ) {
        for item in items {
            match item.what() {
                Ok(What::Message { from, to }) => {
                    let (from, to) = (self.index(from), self.index(to));
                    let sort = if item.sends { Sort::Send } else { Sort::Call };
                    if sort == Sort::Call && !item.refused {
                        open.push((from, to, messages(out)));
                    }
                    out.push(Step::Message(Resolved {
                        from,
                        to,
                        sort,
                        refused: item.refused,
                        label: item.label.as_deref(),
                        answers: None,
                    }));
                }
                Ok(What::Reply { by, to }) => {
                    let by = self.index(by);
                    let (caller, call) =
                        answer(open, &by).expect("validated: a reply answers a call still waiting");
                    let to = to.map_or(caller, |t| self.index(t));
                    out.push(Step::Message(Resolved {
                        from: by,
                        to,
                        sort: Sort::Reply,
                        refused: item.refused,
                        label: item.label.as_deref(),
                        answers: Some(call),
                    }));
                }
                Ok(What::Fragment { operator, operands }) => {
                    through(operator, operands.len(), open, &mut |j, way| {
                        let when = operands[j].when.as_deref();
                        out.push(if j == 0 {
                            Step::Open(operator, when)
                        } else {
                            Step::Operand(when)
                        });
                        self.read(&operands[j].messages, way, out);
                    });
                    out.push(Step::Close);
                }
                // Validation has refused the spec.
                Err(_) => {}
            }
        }
    }
}

/// How many messages the steps so far hold.
fn messages(steps: &[Step<'_>]) -> usize {
    steps
        .iter()
        .filter(|s| matches!(s, Step::Message(_)))
        .count()
}

/// Visits every item, fragments' operands included, in the spec's order,
/// with its JSON pointer.
fn walk<'a>(items: &'a [Item], at: &str, visit: &mut dyn FnMut(&str, &'a Item)) {
    for (i, item) in items.iter().enumerate() {
        let pointer = format!("{at}/{i}");
        visit(&pointer, item);
        if let Ok(What::Fragment { operator, operands }) = item.what() {
            for (j, operand) in operands.iter().enumerate() {
                let inner = operand_at(&pointer, operator, j);
                walk(&operand.messages, &format!("{inner}/messages"), visit);
            }
        }
    }
}

/// The pointer to a fragment's operand: `/messages/3/alt/1`, or
/// `/messages/3/opt` for the one operand of `opt` and `loop`.
fn operand_at(item: &str, operator: Operator, j: usize) -> String {
    match operator {
        Operator::Alt | Operator::Par => format!("{item}/{}/{j}", operator.name()),
        Operator::Opt | Operator::Loop => format!("{item}/{}", operator.name()),
    }
}

/// Checks everything the types cannot express (docs/SPEC.md, Validation).
/// Returns every problem, in the order the spec lists things.
#[must_use]
pub fn validate(seq: &Sequence) -> Vec<SpecError> {
    let mut v = Validator {
        seq,
        errors: top_level(&Common {
            archgram: seq.archgram,
            title: &seq.title,
            description: &seq.description,
            shown_width: seq.shown_width,
            palette: &seq.palette,
        }),
        used: BTreeSet::new(),
    };
    v.texts();
    let ids = v.participants();
    if seq.messages.is_empty() {
        v.error("/messages", "a sequence needs at least one message".into());
    }
    let mut open = Vec::new();
    v.items(&seq.messages, "/messages", &ids, &mut open);
    for (i, p) in seq.participants.iter().enumerate() {
        if ids.contains(p.id.as_str()) && !v.used.contains(p.id.as_str()) {
            v.error(
                &format!("/participants/{i}"),
                format!(
                    "participant `{}` is in no message; take it out, or add the messages it takes part in",
                    p.id
                ),
            );
        }
    }
    v.errors.extend(source_forms(&seq.owned()));
    v.errors
}

/// Each `tech` that `logos` does not carry, located in the spec.
#[must_use]
pub fn check_logos(seq: &Sequence, logos: &dyn crate::logos::Logos) -> Vec<SpecError> {
    let techs = seq.participants.iter().enumerate().filter_map(|(i, p)| {
        p.tech
            .as_deref()
            .map(|t| (format!("/participants/{i}/tech"), t))
    });
    techs_carried(techs, logos)
}

/// A call waiting for its reply: who called whom.
type Call<'a> = (&'a str, &'a str, ());

struct Validator<'a> {
    seq: &'a Sequence,
    errors: Vec<SpecError>,
    /// The participants some message names.
    used: BTreeSet<&'a str>,
}

impl<'a> Validator<'a> {
    fn error(&mut self, pointer: &str, message: String) {
        self.errors.push(SpecError::at(pointer, message));
    }

    /// Every text the drawing carries holds only characters XML allows.
    fn texts(&mut self) {
        let s = self.seq;
        let mut texts: Vec<(String, &str)> = vec![
            ("/title".into(), &s.title),
            ("/description".into(), &s.description),
        ];
        for (i, p) in s.participants.iter().enumerate() {
            texts.push((format!("/participants/{i}/label"), &p.label));
            if let Some(note) = &p.note {
                texts.push((format!("/participants/{i}/note"), note));
            }
        }
        walk(&s.messages, "/messages", &mut |pointer, item| {
            if let Some(label) = &item.label {
                texts.push((format!("{pointer}/label"), label));
            }
            if let Ok(What::Fragment { operator, operands }) = item.what() {
                for (j, operand) in operands.iter().enumerate() {
                    if let Some(when) = &operand.when {
                        texts.push((format!("{}/when", operand_at(pointer, operator, j)), when));
                    }
                }
            }
        });
        for (pointer, text) in texts {
            if let Some(e) = xml_problem(pointer, text) {
                self.errors.push(e);
            }
        }
    }

    /// The participants' ids, each checked for its form and uniqueness, and
    /// their labels and logos' slugs.
    fn participants(&mut self) -> BTreeSet<&'a str> {
        let seq: &'a Sequence = self.seq;
        if seq.participants.len() < 2 {
            self.error(
                "/participants",
                "a sequence needs at least two participants: it shows the messages between them"
                    .into(),
            );
        }
        let mut ids = BTreeSet::new();
        for (i, p) in seq.participants.iter().enumerate() {
            let id = p.id.as_str();
            if !is_valid_id(id) {
                self.error(&format!("/participants/{i}/id"), format!("`{id}` is not a valid id; use lowercase letters, digits and single hyphens, such as `api` or `auth-service`"));
            }
            if !ids.insert(id) {
                self.error(
                    &format!("/participants/{i}/id"),
                    format!("the id `{id}` is used by two participants"),
                );
            }
            if p.label.trim().is_empty() {
                self.error(
                    &format!("/participants/{i}/label"),
                    format!("participant `{id}` has an empty label"),
                );
            }
            if let Some(tech) = &p.tech
                && let Some(message) = tech_form(tech)
            {
                self.error(&format!("/participants/{i}/tech"), message);
            }
        }
        ids
    }

    /// The problem with `id` when no participant has it, the nearest
    /// suggested; and `id` counted as used when one has.
    fn participant(&mut self, ids: &BTreeSet<&str>, id: &'a str) -> Option<String> {
        if ids.contains(id) {
            self.used.insert(id);
            return None;
        }
        Some(match nearest(id, ids.iter().copied()) {
            Some(near) => format!("no participant has the id `{id}`; did you mean `{near}`?"),
            None => format!("no participant has the id `{id}`"),
        })
    }

    /// The items of one list, in time's order: each a message, a reply or
    /// a fragment, its participants named, each reply answering a call
    /// still open. `open` holds the calls waiting for their replies.
    fn items(
        &mut self,
        items: &'a [Item],
        at: &str,
        ids: &BTreeSet<&str>,
        open: &mut Vec<Call<'a>>,
    ) {
        for (i, item) in items.iter().enumerate() {
            let pointer = format!("{at}/{i}");
            match item.what() {
                Err(message) => self.error(&pointer, message),
                Ok(What::Message { from, to }) => {
                    let mut known = true;
                    for (field, id) in [("from", from), ("to", to)] {
                        if let Some(message) = self.participant(ids, id) {
                            self.error(&format!("{pointer}/{field}"), message);
                            known = false;
                        }
                    }
                    // A call waits for its reply, unless it is refused at
                    // once; a send waits for nothing.
                    if known && !item.sends && !item.refused {
                        open.push((from, to, ()));
                    }
                }
                Ok(What::Reply { by, to }) => self.reply(&pointer, ids, open, by, to),
                Ok(What::Fragment { operator, operands }) => {
                    self.fragment(&pointer, ids, open, operator, operands);
                }
            }
        }
    }

    /// A reply answers the latest call to its participant still open, and
    /// goes back to that call's caller.
    fn reply(
        &mut self,
        pointer: &str,
        ids: &BTreeSet<&str>,
        open: &mut Vec<Call<'a>>,
        by: &'a str,
        to: Option<&'a str>,
    ) {
        if let Some(message) = self.participant(ids, by) {
            self.error(&format!("{pointer}/reply"), message);
            return;
        }
        if let Some(to) = to
            && let Some(message) = self.participant(ids, to)
        {
            self.error(&format!("{pointer}/to"), message);
            return;
        }
        let Some((caller, ())) = answer(open, &by) else {
            self.error(
                &format!("{pointer}/reply"),
                format!("`{by}` replies, but no call to `{by}` is waiting for a reply before it"),
            );
            return;
        };
        if let Some(to) = to
            && to != caller
        {
            self.error(
                &format!("{pointer}/to"),
                format!(
                    "the reply from `{by}` answers `{caller}`'s call, so it goes to `{caller}`, not `{to}`; leave `to` out or name `{caller}`"
                ),
            );
        }
    }

    /// A fragment's operands, each with the guards its operator takes, and
    /// the calls left open after it: for `par` its operands run one after
    /// another; for `alt`, `opt` and `loop` a call stays open when any way
    /// through leaves it open, so a later reply is not refused for one way.
    fn fragment(
        &mut self,
        pointer: &str,
        ids: &BTreeSet<&str>,
        open: &mut Vec<Call<'a>>,
        operator: Operator,
        operands: &'a [Operand],
    ) {
        let name = operator.name();
        if matches!(operator, Operator::Alt | Operator::Par) && operands.len() < 2 {
            self.error(
                &format!("{pointer}/{name}"),
                format!("`{name}` needs two operands or more; with one, use `opt`"),
            );
        }
        let last = operands.len().saturating_sub(1);
        for (j, operand) in operands.iter().enumerate() {
            let here = operand_at(pointer, operator, j);
            self.guard(&here, operator, operand, j == last);
            if operand.messages.is_empty() {
                self.error(
                    &format!("{here}/messages"),
                    format!("an operand of `{name}` holds no messages"),
                );
            }
        }
        through(operator, operands.len(), open, &mut |j, way| {
            let here = operand_at(pointer, operator, j);
            self.items(&operands[j].messages, &format!("{here}/messages"), ids, way);
        });
    }

    /// An `alt` operand's guard is required, and `else` only on its last;
    /// `opt` and `loop` may take one, never `else`; a `par` operand takes
    /// none.
    fn guard(&mut self, here: &str, operator: Operator, operand: &Operand, last: bool) {
        let name = operator.name();
        let when = operand.when.as_deref().map(str::trim);
        match (operator, when) {
            (_, Some("")) => self.error(
                &format!("{here}/when"),
                "the guard is empty; say when the operand happens, or leave `when` out".into(),
            ),
            (Operator::Alt, None) => self.error(
                here,
                "each operand of `alt` needs a guard in `when`; the last may be `else`".into(),
            ),
            (Operator::Alt, Some("else")) if !last => self.error(
                &format!("{here}/when"),
                "`else` goes on the last operand of `alt`".into(),
            ),
            (Operator::Opt | Operator::Loop, Some("else")) => self.error(
                &format!("{here}/when"),
                format!("`else` belongs to `alt`; `{name}` has one operand"),
            ),
            (Operator::Par, Some(_)) => self.error(
                &format!("{here}/when"),
                "the operands of `par` all happen, so they take no guard".into(),
            ),
            _ => {}
        }
    }
}
