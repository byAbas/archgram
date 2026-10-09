# Feature: more kinds of diagram, each grounded in its standard

| Field   | Value      |
|---------|------------|
| Version | 0.2        |
| Date    | 2026-10-09 |
| Status  | Draft      |
| Release | Planned, sequence first |
| Issue   | [#126](https://github.com/byAbas/archgram/issues/126) |

What this feature does and why, in more detail than `docs/PRD.md`, which
points here. Each kind gets a document of its own, written from the
standard named for it below, before its work starts; the fields each adds
go into `docs/SPEC.md`, and its symbols into DESIGN.md, as it is built.

## Problem

archgram draws one kind of diagram, the architecture of a system: parts,
what calls what, and the way a request goes (PRD §6, §7). Readers also
need the order of the calls between parts, the states a thing goes
through, the steps a process takes across people, and where data goes.
Each has a diagram whose meaning people already share, and drawn as
architecture it loses that meaning.

### Evidence

- On a frontend system design reference site, the same one as in
  [what backs a diagram](sources.md#problem), 20 of its 48 Mermaid
  diagrams are sequence diagrams.
- An OAuth request drawn as architecture put 8 messages onto 6 arrows,
  and the order was lost in the still image.
- archify (tt-a1i/archify, v3.0.1) draws five kinds: architecture,
  workflow, sequence, dataflow, lifecycle (`archify/SKILL.md`, Type
  router). Its renderers take positions from the agent (`grid.mjs`: "Not
  auto-layout — fixed cell math only"), and its open issues ask for
  automatic layers (#359) and first-diagram reliability (#344).
- What archgram already covers, tried on 2026-10-08: dataflow almost
  fully, workflow in part (frames as lanes, `stop` as a refusal),
  lifecycle not at all (a transition back to its own node is refused),
  sequence not at all (a non-goal, PRD §9).

## Why a standard for each kind

archgram draws one kind of diagram today, the architecture of a system,
and its meaning is archgram's own: parts, what calls what, the way a
request goes (PRD §6, §7). A new kind has a meaning people already share:
a sequence diagram, a state machine, a process with lanes and decisions, a
data flow diagram. Readers learned those meanings from a standard or from
the books that defined them, so archgram takes each new kind's meaning
from there and from nowhere else.

- **The standard is the source of truth for meaning**: what the parts of
  the diagram are, what each one means, what is valid, and the symbols
  readers already know (a diamond for a decision, a filled circle for a
  start). The implementation plan of each kind cites the clauses it
  implements.
- **archgram's principles still hold** (PRD §4): reader first, semantic
  and never coordinates, deterministic, monochrome with colour only where
  it carries meaning, accessible. Where the standard and a principle
  differ, the principle decides how it looks, never what it means.
- **A subset, named.** archgram draws the part of each standard most
  readers use, lists what it leaves out, and never claims conformance. The
  standards define conformance for tools that support all of a set
  (BPMN 2.0.2, 2.2, "All the elements in the sub-class MUST be
  supported"); archgram supports fewer.
- **Principle 4 is unchanged.** It is about the engine: layout and routing
  stay archgram's own, and papers and other engines are examples, not the
  source of truth. The standards here decide meaning, not code.

### The sources

| Kind | Source of truth | Where |
|---|---|---|
| Sequence | OMG Unified Modeling Language 2.5.1, formal/17-12-05, December 2017 | Clause 17, Interactions: 17.3 Lifelines, 17.4 Messages, 17.6 Fragments, 17.8 Sequence Diagrams. https://www.omg.org/spec/UML/2.5.1/PDF |
| Lifecycle | OMG Unified Modeling Language 2.5.1 | Clause 14, State Machines: 14.2 Behavior StateMachines, 14.2.4 Notation, 14.5.7 PseudostateKind, 14.5.12 TransitionKind |
| Workflow | OMG Business Process Model and Notation 2.0.2, formal/2013-12-09 | 2.2 Process Modeling Conformance, Table 2.1 (Descriptive Conformance Sub-Class); 9.3.2 Lanes; 10.3 Activities; 10.5 Events; 10.6 Gateways. https://www.omg.org/spec/BPMN/2.0.2/PDF |
| Dataflow | Tom DeMarco, *Structured Analysis and System Specification*, Yourdon Press, 1978; Chris Gane and Trish Sarson, *Structured Systems Analysis: Tools and Techniques*, 1979 | Not read yet (books, not online); see Open questions |
| Architecture | archgram's own (PRD §6, §7) | Unchanged |

Checked on 2026-10-09: the UML and BPMN pages at omg.org list 2.5.1 and
2.0.2 as their latest formal versions, and the clauses above are from
those PDFs' tables of contents. The DFD books are confirmed only by
library records (https://en.wikipedia.org/wiki/Structured_analysis;
https://find.library.upenn.edu/catalog/991101033503681;
https://bdu.siu.edu.ar/bdu/Record/B-45-UBP01005); their text is not.

## Who it serves

| User | What they get |
|---|---|
| A reader of a README or a design doc | The diagram that answers their question, in symbols they know: the order of calls, the states, the steps, where data goes |
| A developer or a writer | One tool, one spec format and one look for every kind, checked against the code or the document as architecture is |
| An agent | A rule for choosing the kind, and the standard's own vocabulary to write it in |

## What every kind shares

- One spec format, one command, one package. `build` reads the kind from
  the spec.
- Parsing, located errors, sources and `check`, measured text and two-line
  labels, the embedded font, tokens and themes, `--theme-file`,
  `shownWidth`, one self-contained SVG, light and dark, SMIL animation, the
  still image and a screen reader's account.
- Monochrome by archgram's rule: the shape tells the kind of a part, and
  colour belongs to what happens (passing, refused) and to logos.
- Where a standard has a symbol readers know, archgram draws that symbol
  in its own stroke and colours (DESIGN.md decides how); where it has none
  readers know, archgram's card (Decided, 5).

## Sequence (UML 2.5.1, clause 17)

Its own document, with the spec's shape and its decisions:
[sequence.md](sequence.md).

What the standard says, and archgram takes:

- A lifeline is a participant's time-line: "In an interaction diagram a
  Lifeline describes the time-line for a process, where time increases
  down the page" (17.3.3). Participants are columns; time runs down.
- A message has a sort: `synchCall`, `asynchCall`, `asynchSignal`,
  `createMessage`, `deleteMessage`, `reply` (17.12.22, MessageSort).
  archgram takes a call (synchronous), a send (asynchronous, both
  asynchronous sorts as one) and a reply, which "has a dashed line with
  either an open or filled arrow head" (17.4.4).
- A combined fragment has an operator: `seq`, `alt`, `opt`, `break`,
  `par`, `strict`, `loop`, `critical`, `neg`, `assert`, `ignore`,
  `consider` (17.12.15, InteractionOperatorKind). archgram takes `alt`
  (one of several, each with a guard), `opt` (it happens or not), `loop`
  and `par`.
- A message to the same lifeline (a self-call).

Left out: creation and deletion of lifelines, the other operators,
interaction uses, gates, execution specifications (activation bars, to be
decided), timing constraints, communication, interaction overview and
timing diagrams.

archgram's own:

- Participants are written in the sequence spec itself, as nodes are,
  each with a kind, a logo and a source; the spec stands alone and reads
  no architecture spec (Decided, 2).
- A message may name the code that sends it (`source`), as an edge does.
- A refused message: a call answered with a refusal, drawn as a flow's
  refusal is (PRD §6.4). UML has no refusal; this is archgram's addition
  (Decided, 4).
- Animation: the messages in order, one signal at a time, a `par`'s
  messages together.
- Still image: each message numbered in order (the sequence's own order
  is the meaning, so numbers are on by default).

Validation: a message names participants that exist; a reply answers a
call before it; an `alt`'s operands each have a guard; fragments nest
and do not cross.

## Lifecycle (UML 2.5.1, clause 14)

What the standard says, and archgram takes:

- States, and transitions between them, each with an optional trigger,
  guard and effect written on its line (14.2).
- Pseudostates: `initial`, `deepHistory`, `shallowHistory`, `join`,
  `fork`, `junction`, `choice`, `entryPoint`, `exitPoint`, `terminate`
  (14.5.7, PseudostateKind). archgram takes `initial`, "shown as a small
  solid filled circle", and `choice`, "shown as a diamond-shaped symbol"
  (14.2.4.6).
- The final state: "a circle surrounding a small solid filled circle"
  (14.2.4.5), "signifying that the enclosing Region has completed"
  (14.2.3.6).
- A transition back to an earlier state, and one to the same state (a
  self-transition; 14.5.12, TransitionKind).
- A composite state as a frame around its states, one level (to be
  decided).

Left out: history, fork and join, junction, entry and exit points,
terminate, orthogonal regions, protocol state machines, internal
transitions and entry, exit and do behaviours inside a state.

archgram's own:

- To decide in lifecycle's own document (Decided, 4): a final state that
  says how the thing ended, succeeded or failed, in the passing and
  refusal colours; a waiting state, as archify has. UML has neither.
- Animation: a run, a path through the states, as a flow is, a retry
  taking the transition back.
- Still image: as architecture.

Validation: from UML, "An initial Vertex can have at most one outgoing
Transition" and its transition has no trigger or guard (14.5.6.7), and "A
FinalState cannot have any outgoing Transitions" (14.5.2.5). archgram's
own, for a reader: one initial pseudostate; every state reachable from
it; a choice with two outgoing transitions at least, each with a guard.

## Workflow (BPMN 2.0.2)

What the standard says, and archgram takes, from the Descriptive
Conformance Sub-Class (2.2.2, Table 2.1): lanes (`laneSet`; "A Lane is a
sub-partition within a Process", 7.3.1; 9.3.2), unconditional sequence flow, the none
start and end events, tasks (none, user, service), the exclusive gateway
and the parallel gateway (10.6.2, 10.6.4), a data store reference and a
text annotation.

Left out (in Table 2.1, so archgram claims no Descriptive conformance):
pools and message flow between them, call activities, expanded and
collapsed sub-processes, data objects. Left out besides: intermediate
events, inclusive, complex and event-based gateways, compensation,
choreographies.

archgram's own:

- A lane is a band across the whole diagram, its name at its start, and
  each node stays in its lane; the flow runs along the lanes.
- A task may name its technology's logo and the code that runs it.
- A refused step: the exclusive gateway's other branch, or a `stop` as
  today (PRD §6.4).
- Animation: a flow is a run through the process, a parallel gateway's
  branches together.

Validation: one start event at least, an end event reachable from every
node; a gateway that splits has two outgoing flows at least; each node in
one lane.

## Dataflow (structured analysis)

What the books say (to be confirmed from the text; see Open questions):
a data flow diagram has four parts, the process, the data flow, the data
store and the external entity (DeMarco calls it the terminator), and a
flow carries data, named as data, not as a call.

archgram takes the four, mapped onto the kinds it has: external entities
are `users`, `browser`, `mobile`, `desktop` or an external node; processes
are `service`, `script`, `agent`, `model`, `tool`; stores are `database`,
`storage`, `cache`, `queue`, `vector-store`, `file`.

Left out: levelled diagrams (a process opened into its own diagram),
which a reader gets as separate specs; the data dictionary.

archgram's own:

- A flow's label names the data, a noun. Whether it may carry a
  classification (personal data), which the books do not have, is decided in dataflow's own document (Decided, 4).
- A stage, a frame named for a step of a pipeline (extract, transform,
  load).

Validation (rules the books are said to give, to be confirmed): a flow
starts or ends at a process; no flow joins two stores, or two external
entities, directly.

## Choosing the kind (the skill)

The skill chooses the kind from the question the reader asks:

| The reader asks | Kind |
|---|---|
| What is it made of, and what calls what? | Architecture |
| In what order do they call each other, and what comes back? | Sequence |
| What states does it go through? | Lifecycle |
| What steps does the work take, and who does each? | Workflow |
| Where does the data go, and where is it kept? | Dataflow |

When a request fits a kind archgram does not draw yet, the skill says so
rather than drawing it as another kind: drawn as architecture, the OAuth
request above lost its order without a word.

## Limits

- A subset of each standard: a reader who needs history states, message
  flows between pools or levelled data flow diagrams needs a modelling
  tool.
- A sequence diagram grows down with its messages; past about 30 messages
  (to be measured) it is split, as an architecture diagram is past about
  10 nodes and 12 edges.
- The standards decide meaning; how each symbol is drawn is DESIGN.md's,
  so a diagram looks like archgram's, not like a UML tool's.

## Success criteria

- A spec without a kind draws the same bytes as before; the goldens do
  not change.
- Each kind: goldens for a test set; byte-identical output across runs
  and systems; no overlap; WCAG 2.1 AA in both themes; the still image
  and a screen reader carry the order.
- Sequence: the OAuth request above draws each of its 8 messages on its
  own arrow, in order, in the still image; the reference site's 20
  sequence diagrams are redrawn and a reviewer accepts them.
- Lifecycle: a retry (a transition back) and a self-transition draw
  without crossing a state.
- Workflow: every node in its lane; lanes span the drawing.
- The skill: an evaluation case for each kind, and one where the request
  fits a kind archgram does not draw, passing at the usual bar (PRD §8).

## Decided (2026-10-09)

1. **A spec says its kind with a top-level field, `diagram`**:
   `diagram: sequence`, `diagram: lifecycle`, `diagram: workflow`,
   `diagram: dataflow`, `diagram: architecture`. Absent, it is
   `architecture`, so every spec written today draws the same bytes.
   (`kind` stays the node's.)
2. **A sequence is a spec of its own** (`diagram: sequence`), with its own
   participants, standing alone. It does not read an architecture spec,
   and the two are not tied together; a participant named like a node is
   only a name. Sharing participants with an architecture spec is left
   for later, if readers ask for it.
3. **Sequence comes first**, then dataflow, workflow and lifecycle. It is
   the largest of the four (a new layout, its rendering and its
   animation), but it is where the demand is: 20 of the reference site's 48
   diagrams are sequence diagrams, and an OAuth request drawn as
   architecture lost its order. The `diagram` field arrives with it.
4. **A refused message stays**, archgram's addition to UML, which has no
   refusal: a call answered with a refusal, drawn as a flow's refusal is
   (PRD §6.4), since whether a request is let through is the question of
   most auth and validation sequences. The other additions (a final
   state's outcome, a waiting state, a flow's data classification) are
   decided in their kinds' own documents, each kept only if it can say
   why a reader needs it.
5. **Symbols: the standard's where readers know it, archgram's cards for
   the rest.** A dashed reply arrow (UML 17.4.4), a filled circle for the
   initial pseudostate and a bull's eye for the final state (14.2.4.6,
   14.2.4.5), a diamond for a choice or a gateway (14.2.4.6; BPMN 10.6),
   lanes as bands (BPMN 9.3.2), each drawn in archgram's own stroke and
   colours as DESIGN.md sets them. Participants, states, tasks and
   processes stay archgram's cards, with their icons and logos, so a
   diagram reads as archgram's, not a UML tool's.

## Open questions

- The DFD books' own text: the four elements' names and the validation
  rules above are from secondary sources (a Yourdon course document and
  course notes citing DeMarco). Read DeMarco 1978 before the dataflow
  feature document, or choose a source that is online.
- Whether to take activation bars in sequence (UML: ExecutionSpecification,
  17.2.4.4, 17.12.8) in the first version.
- Whether workflow's lanes and architecture's frames are one thing in the
  engine with two looks, or two.

## Changelog

| Version | Date       | Change |
|---------|------------|--------|
| 0.1     | 2026-10-09 | First draft, from #126: sequence and lifecycle from UML 2.5.1, workflow from BPMN 2.0.2, dataflow from structured analysis; a named subset of each, no conformance claimed; `diagram` names a spec's kind; a sequence is a spec of its own; sequence first; a refused message kept; the standard's symbols where readers know them. |
| 0.2     | 2026-10-09 | Sequence has its own document, [sequence.md](sequence.md). |
