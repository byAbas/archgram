# Feature: sequence diagrams

| Field   | Value      |
|---------|------------|
| Version | 0.5        |
| Date    | 2026-10-09 |
| Status  | Draft      |
| Release | Planned, the first of the new kinds |
| Issue   | [#126](https://github.com/byAbas/archgram/issues/126) |

What this kind does and why, in more detail than
`docs/features/diagram-kinds.md`, which names its source of truth and the
decisions shared by every kind. Its fields go into `docs/SPEC.md` and its
symbols into DESIGN.md as it is built.

## Problem

An architecture diagram says what calls what, not in what order, nor what
comes back. A sign-in, an OAuth exchange, a cache miss or a payment is a
conversation: the order of its messages and the answers are the point.
Drawn as architecture, an OAuth request put its 8 messages onto 6 arrows,
and the still image lost their order. On a frontend system design
reference site, 20 of 48 diagrams are sequence diagrams, which archgram
cannot draw today ([diagram-kinds.md](diagram-kinds.md#evidence)).

## Who it serves

| User | What they get |
|---|---|
| A reader of a README or a design doc | The order of the calls between the parts, what each returns, and where a request is turned away, in the notation they know from UML |
| A developer | A sequence beside the code, each call tied to the line that makes it, checked when that line goes |
| An agent | A spec it writes in the order the code runs, with the vocabulary of an architecture spec |

## Source of truth

OMG Unified Modeling Language 2.5.1, clause 17, Interactions
(https://www.omg.org/spec/UML/2.5.1/PDF). What archgram takes from it:

- **Lifelines.** "A Lifeline is shown using a symbol that consists of a
  rectangle forming its 'head' followed by a vertical line (which may be
  dashed) that represents the lifetime of the participant" (17.3.4.1).
  "Events on the same time-line are ordered linearly down the page"
  (17.3.3.1).
- **Messages.** Of the six sorts (17.12.22, MessageSort): a synchronous
  call, an asynchronous send (`asynchCall` and `asynchSignal` as one),
  and a reply, which "has a dashed line with either an open or filled
  arrow head" (17.4.4.1). A message to the sender's own lifeline.
- **Combined fragments.** "A solid-outline rectangle. The operator is
  shown in a pentagon in the upper left corner of the rectangle"; operands
  "separated by a dashed horizontal line" (17.6.4.3, 17.6.4.1). Of the
  twelve operators (17.12.15), `alt`, `opt`, `loop` and `par`.
- **Guards.** "Shown in square brackets covering the lifeline where the
  first event occurrence will occur", `[else]` among them; omitted, true
  is assumed (17.6.4.2).
- **Activations.** "ExecutionSpecifications are represented as thin
  rectangles (gray or white) on the lifeline", and overlapping ones "by
  overlapping rectangles" (17.2.4.4, Figure 17.2).

Left out: creating and deleting lifelines, the other eight operators, interaction
uses, gates, lost and found messages, timing constraints, and the
communication, interaction overview and timing diagrams.

## Requirements

### Spec

A spec names its kind, `diagram: sequence`, and holds `participants` and
`messages`; time is the order of the `messages` list.

```yaml
archgram: 1
diagram: sequence
title: sign in
description: The browser signs in through the API, which checks the password and returns a session.
participants:
  - { id: browser, kind: browser, label: Browser }
  - { id: api, kind: service, label: API, tech: nodedotjs, source: ../../src/api.ts }
  - { id: db, kind: database, label: Users, tech: postgresql }
messages:
  - phase: Sign in
  - { from: browser, to: api, label: POST /login }
  - phase: Check the password
  - { from: api, to: db, label: find user, source: "../../src/api.ts#db.users.find(" }
  - { reply: db, label: user }
  - alt:
      - when: password matches
        messages:
          - { reply: api, label: session, to: browser }
      - when: else
        messages:
          - { reply: api, label: 401, to: browser, refused: true }
```

- `participants` are written as an architecture spec's `nodes`: `id`,
  `kind`, `label`, `note`, `tech`, `variant`, `source`. A sequence spec
  stands alone and reads no architecture spec.
- A message has `from`, `to` and `label`; `async: true` makes it a send
  that does not wait; `source` as an edge's.
- `reply: <sender>` answers the latest call to that sender not yet
  answered; `to` may be left out when that call makes it plain.
- `alt`, `opt`, `loop` and `par` are items of the list, each holding
  operands, each operand its own `messages`; fragments nest. Each `alt`
  operand has a guard in `when`, `else` only on the last; `opt` and
  `loop` take one operand with an optional `when`; `par` takes two or
  more operands.
- `refused: true` marks a message that turns the request away, archgram's
  addition to UML (diagram-kinds.md, Decided 4).
- `look` picks how it is drawn: `cards`, the default, or `avatars`
  (Looks, below).
- `phase: <name>` is an item of the top-level list that starts a phase:
  the messages after it, up to the next phase, are that phase, a step of
  the story told in a few words ("Sign in", "Check the password"). It
  divides the list as PlantUML's `== name ==` divides a diagram into
  logical steps (plantuml.com/sequence-diagram, "Divider or separator");
  UML has no such thing, and it is archgram's addition. Messages before
  the first phase belong to none.
- The fields of an architecture spec that do not apply are errors here:
  `nodes`, `edges`, `frames`, `flows`, `hints`, `direction`, and, since a
  sequence moves in its own way (Animation), `signal`, `border`, `wait`
  and `glow`. `shownWidth`, `card`, `logo`, `palette`, `still`, `credit`
  apply as there.

### Validation

A sequence spec is rejected, every problem at its line and column, when:

- there are fewer than two participants or no message, or a participant
  is in no message;
- an item is not exactly one of a message, a reply and a fragment, or a
  fragment carries a message's fields;
- a message names a participant that does not exist;
- a reply has no call to that participant before it still waiting, or
  names a `to` other than that call's caller; a send that does not wait,
  and a refused call, wait for no reply. Inside an `alt`, `opt` or
  `loop`, each way through starts from the calls waiting before it, and
  a call left waiting by any way may be answered after it; a `par`'s
  operands run one after another;
- an `alt` or a `par` has fewer than two operands, an operand of `alt`
  lacks `when`, `else` is on an operand other than an `alt`'s last, a
  `par` operand has a guard, a guard is empty, or an operand holds no
  messages;
- a `phase` is inside a fragment, carries any other field, has an empty
  name, or no message follows it before the next phase or the end.

An operand may name the code that makes its choice in `source`, as a
message does.

`archgram check` holds every source to the code as in architecture
([sources.md](sources.md)).

### Layout

- Participants are columns, left to right in the order the spec lists
  them; their heads in one row at the top, a thin dashed lifeline down
  from each to the foot of the diagram (UML 17.3.4.1).
- Each phase is a band across the page over its messages' rows, its name
  at its top left in small capitals, with a row of its own above its first
  message. Every band is filled alike, a gap between two: a tint behind
  every other one would leave a connector short of 3:1 in light.
- Each message is a row of its own, in the list's order, so nothing
  happens at the same height except inside a `par`, whose operands still
  take rows of their own. Time runs down the page; there is no
  `direction`.
- The space between two columns is as wide as the widest label between
  them needs, measured as archgram measures every label, a long label on
  two lines (DESIGN.md, Components: Edge label).
- A message to the same participant is a short loop out to the right of
  its lifeline and back half a row lower, so it takes a row and a half.
- A fragment's frame covers the columns of the lifelines its messages
  touch and the rows they take. Its operator and its first guard are one
  pill on its top edge, `alt · password matches`; its operands are
  divided by dashed lines, each later guard a pill on its line, `else`.
  The pill leaves a row's room between it and the messages above and
  below. Nested frames sit inside with a margin.
- In `cards`, a call's receiver is active from the call to the reply that
  answers it, a thin bar on its lifeline (17.2.4.4):
  - when an `alt` answers a call in each of its operands, the bar runs
    to the last of those replies;
  - a call never answered keeps its bar to the last message its receiver
    sends before its caller calls it again, and at least half a row; a
    call to itself never answered, half a row;
  - a send and a refused call start no bar, since nothing waits on them;
  - a call to a participant already active, a call to itself among them,
    draws a second bar over the first, moved to its right, as UML's
    overlapping rectangles (Figure 17.2).
- Width is held to where the diagram is shown, as in architecture
  ([shown-width.md](shown-width.md)): `build` warns when the participants
  need more width than keeps the text readable, and says to drop or merge
  participants or split the diagram.

### Rendering

- A participant's head is drawn at the top of its lifeline, as its look
  draws it (Looks).
- A call is a solid line with a filled arrowhead; a send that does not
  wait, an open arrowhead; a reply, a dashed line with an open arrowhead
  (UML 17.4.4.1; Decided 8). The label sits above its line, centred
  between the two lifelines, as plain text led by its number. A message
  to itself has its label beside its loop.
- The main path reads first: a call's line and label are in the text's
  colour, a reply's quieter, its line thinner and its label muted; a send
  is drawn as a call. Monochrome still: weight and shade, not hue, make
  the difference.
- A refused message ends in a ✕ at the participant that refuses, in the
  refusal colour, as a refused flow does (PRD §6.4); its label is in that
  colour too.
- Monochrome, both themes, the embedded font, one self-contained SVG, as
  every kind.

### Looks

`look` picks one of two. Both draw the same spec into the same columns,
rows, frames and order; only these differ. Samples of both, in both
themes, are in [docs/samples/sequence/](../samples/sequence/).

| | `cards`, the default | `avatars` |
|---|---|---|
| Head | archgram's card: the icon in its badge, the label beside it, the logo in the corner | A circle with the icon, the logo in a small badge on its edge, the label under it |
| Activation | A thin, light bar on the lifeline, from a call to its reply | None |
| Fragment frame | Solid, rounded | Dashed, rounded |
| Operator and guard | A pill on the frame's top edge, at the left | A pill at the middle of the frame's top edge |

An external participant's head has a dashed edge in both, as an external
node's card does.

Where both looks part from UML's notation, as archgram's own drawing of
its symbols (diagram-kinds.md, Decided 5):

- The operator's pentagon (17.6.4.3) is a pill, and the guard's square
  brackets (17.6.4.2) are left out: the pill bounds the guard. The
  operator's name stays, so `alt`, `opt`, `loop` and `par` read as UML's.
- Phases are archgram's, drawn as bands across the page.

### Animation

The drawing stays readable the whole time: motion points at a message,
it never hides one. Samples are in
[docs/samples/sequence/](../samples/sequence/) (`motion`).

- One phase plays at a time. Its name darkens; the other phases' messages
  dim to 60% at most, never further, and the heads stay as they are.
  Messages before the first phase play as a phase of their own.
- Its messages play in the list's order. A message's line is drawn from
  its sender, darker than at rest, and its arrowhead takes that colour
  the moment the line reaches it, so line and head change as one. The
  line stays dark until its phase ends.
- While its phase plays, a faint tint grows across the phase's band
  between the two lifelines each message joins, as the message goes,
  and keeps what it has covered until the phase ends.
- An `alt` plays as alternatives, not in sequence: its first way plays,
  then its lines dim back, an "or" appears by the next operand's pill,
  and the next way plays from where the first began, the tint starting
  again with it.
- A `par`'s operands start together and each keeps its own order; an
  `opt` and a `loop` play their operand once, archgram not knowing how
  many times a loop repeats.
- A refused message's line is drawn in the refusal colour, its ✕ at its
  end; what follows plays after it.
- A pause separates one phase from the next, and a rest the end from the
  start again, with the whole story shown.
- Under reduced motion, nothing moves: the still image.

### Still image and screen reader

- The still image numbers each message by its order, on by default: in a
  sequence the order is the meaning. `still: none` turns the numbers off.
- An `alt`'s operands are alternatives, and their numbers say so: each
  message of an operand carries its operand's letter, `4a` to `7a`, then
  `4b`; each later operand starts again from the alt's first number, and
  after the `alt` the count goes on from its longest operand. In nested
  `alt`s, each one's letter in turn (`5ab`).
- A screen reader hears each message in words, in order, after the
  description, a fragment's guard before its messages ("If password
  matches: 4a. API replies to Browser: session."), a phase's name before
  its messages, a refused message ending "refused".

### The skill

- It draws a sequence when the reader asks in what order the parts call
  each other and what comes back (diagram-kinds.md, Choosing the kind).
- From code, it follows one entry point's calls in the order they run,
  each call's source the line that makes it; from a document, each
  message's source the sentence that states it.
- Past about 15 messages, it splits the sequence, by phase or by
  fragment. 15 is a starting value, to be measured (Success criteria).

## Limits

- One diagram, one conversation: a sequence does not show the parts that
  take no part in it, nor how they are deployed; that is architecture's.
- A loop is drawn once; its count, if the code knows it, goes in its
  guard.
- `par` shows what may happen in any order, not timing: no durations, no
  clocks.
- A sequence grows down with its messages; past the split value it is
  hard to follow on one screen, however wide.

## Not in this

- Other looks, or a look for one participant only.
- Lifelines created or destroyed during the conversation.
- The other UML operators, interaction uses and gates.
- Reading an architecture spec's nodes as participants.

## Success criteria

- A spec without `diagram` draws the same bytes as before; the goldens do
  not change.
- Each look has its goldens, and the same spec draws the same columns and
  rows in both.
- The OAuth request draws each of its 8 messages on its own line, in
  order, numbered in the still image.
- On a test set: no two labels overlap, no label crosses a lifeline's
  head, every frame holds its messages; byte-identical output across runs
  and systems; WCAG 2.1 AA in both themes.
- `check` fails, at the spec's line, once the code behind a call is gone.
- The reference site's 20 sequence diagrams are redrawn, and a reviewer
  accepts them; from them the split value is set where the smallest text
  stays readable at a README's width, 880 px.
- The skill: an evaluation case that asks for the order of calls draws a
  sequence, and one that asks what a system is made of still draws
  architecture, at the usual bar (PRD §8).

## Decided (2026-10-09)

1. Activation bars in the `cards` look, from a call to its reply, none in
   `avatars` (Decided 9). Until 0.3: none in the first version.
2. All four fragments, `alt`, `opt`, `loop` and `par`.
3. The spec's shape above: `participants`, and `messages` in time's
   order, fragments as items holding operands; `reply: <sender>`;
   `when` for a guard; `refused: true`.
4. The skill splits past about 15 messages, a starting value measured
   once the engine draws.
5. Sources as in architecture: a participant's file, a call's line,
   optional on a reply and a guard; a document's sentence when drawn from
   one.
6. A participant's head is drawn once, at the top of its lifeline, as
   UML draws it (17.3.4.1) and archify does; a sequence past about 15
   messages is split, so it stays within a screen.
7. No rule of its own for an `alt` whose operands all refuse, or whose
   guards none hold: its operands play in turn, each refusal travelling
   back before the next starts, and an `alt` without `else` plays every
   operand, since the drawing shows the possible paths, not one run. The
   still image carries the same: the frame, the guards, the numbers.
   archify has no fragments to compare (an acknowledged gap, its issues
   #312 and #93).
8. Arrowheads keep UML's distinction (17.4.4.1): a call's filled, a
   send's and a reply's open, a reply's line dashed. The shape, not a
   colour, says whether a message waits.

9. Its bands and label pills replaced by 12. Two looks, chosen from samples ([docs/samples/sequence/](../samples/sequence/)):
   `look: cards | avatars`, `cards` by default, its activation bars
   UML's (17.2.4.4). The field is `look`, not `style`, which an edge
   already uses for its line (docs/SPEC.md, Edges).

10. Replaced by 13. In `cards`, a bar fills down as its call is answered, rather than
    lighting at once or not at all: it shows who is waiting, and for how
    long. archify's trace leaves its bars still (its
    `renderers/sequence/render-sequence.mjs`).

11. Phases, `phase: <name>` items of the top-level list, each starting a
    phase up to the next one, as PlantUML's divider does; drawn as bands
    across the page, named at their top left. Chosen from samples, over a
    gutter of names on the left.
12. A lighter drawing, after the owner found the first one hard to follow:
    thin dashed lifelines in place of bands, labels as plain text above
    their lines, and a call's line and label stronger than a reply's.
    Pills stay for a fragment's operator and guards. This replaces the
    band and the label pills of Decided 9; the two looks stay.
13. Motion phase by phase, each line drawn with its arrowhead lit as it
    arrives, a faint tint between the lifelines it joins, an `alt`'s ways
    as alternatives, numbered `4a`, `4b`; nothing dimmed below 60%. This
    replaces the lit heads, pills and filling bars of 0.4 (Decided 10),
    which lit too much at once, and the refusal's way back. Chosen from
    samples.

## Open questions

None.

## Changelog

| Version | Date       | Change |
|---------|------------|--------|
| 0.1     | 2026-10-09 | First draft, from #126 and UML 2.5.1 clause 17: lifelines, calls, sends and replies, `alt`, `opt`, `loop`, `par`; `diagram: sequence` with `participants` and `messages`; a refused message; no activation bars yet; split past about 15 messages, to be measured; sources as in architecture. |
| 0.2     | 2026-10-09 | The heads drawn once, at the top; no rule of its own for an `alt` that refuses throughout; UML's arrowheads; the validation as `check` holds it: two participants and a message at least, one form per item, a refused call waiting for no reply, the calls waiting through a fragment; an operand may name its source. |
| 0.3     | 2026-10-09 | Two looks, `cards` and `avatars`, from samples; each column a band; labels and guards in pills, the operator named; activation bars in `cards`, from a call to its reply. |
| 0.4     | 2026-10-10 | In `cards`, a bar fills down as its call is answered. |
| 0.5     | 2026-10-10 | Phases as dividers, drawn as bands; thin lifelines, labels as plain text, a call stronger than a reply; motion phase by phase, an `alt`'s ways as alternatives numbered `4a`, `4b`; the bars no longer fill, and a refusal no longer travels back. |
