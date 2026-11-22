# PRD: archgram

| Field   | Value      |
|---------|------------|
| Version | 0.45       |
| Date    | 2026-11-22 |
| Status  | Draft      |
| Owner   | Abas Turabli |

This document describes what archgram is and why it exists; which
document holds every other fact is in `CONTRIBUTING.md` (Each fact lives
in one document). A feature that needs more than a few lines here has
its own document in `docs/features/`, with its problem, requirements,
limits and success criteria; this one says what it is and points there.
If a requirement changes, this document, or the feature's own, changes
first.

## 1. Problem

A software architecture diagram in a README is read in about thirty seconds
by someone who did not build the system. Today it is made in one of two
ways, and both fail that reader.

Drawn by hand in Figma, draw.io or Excalidraw, it looks good on the day it
is made and drifts from the code afterwards, because every change means
opening a tool and moving boxes. Generated from Mermaid or a similar text
format, it stays in the repository, but the output is generic: no visual
vocabulary for what a box is, weak layout for nested groups such as a VPC
or a trust boundary, no dark mode of its own, no way to show the order in
which things happen.

A diagram kept as text beside the code is cheap to change, but nothing
says when it must change. A box still names a module deleted a month
ago, a line still shows a call no longer made, and the diagram stays
plausible and wrong. With coding agents changing a project many times a
day, a diagram goes stale faster than anyone reads it closely enough to
notice. The same holds for a diagram drawn from a document rather than
from code, a pattern on a docs site or a system in a design doc or an
engineering blog post: nothing says when the text beside it changes.

When the diagram does change with the code, a pull request shows it as
changed lines of a spec, and the code as thousands more. What the change
does to the architecture, a new service, a call no longer made, is left
for the reviewer to work out, more and more often for a change an agent
wrote.

A system is not only its parts. Readers also need the order of the calls
between them, the states a thing goes through, the steps a process takes
across people, and where data goes. Each has a diagram whose meaning
people already share, from UML, BPMN or structured analysis. Drawn as an
architecture diagram they lose that meaning: an OAuth request's eight
messages on six arrows lose their order.

AI coding agents hit the same wall from the other side. Asked for a
diagram, an agent writes the SVG by hand, every coordinate and every
path. In a test of three diagram tasks, agents doing so took 6.3 minutes
per diagram on average, and 9.3 minutes with a skill that added previews
and self-critique; how that time split between layout and the rest was
not measured.

Without a skill, an agent today no longer draws by hand. In the skill's
first evaluation (2026-10-07, Claude Opus 5.5, eight cases, one run each,
each graded by an independent agent), the agent without the skill wrote
a Mermaid chart into the README, or a sketch into its reply, in about
half a minute: in none of the project's colours, tied to no line of
code, and checked by nothing afterwards. Asked to update or repair a
diagram, it changed the spec but neither checked it nor drew it again,
so the README kept the old picture. It passed 31 of 71 checks; with the
skill, the same agent passed 64 of 71, in 56 seconds on average against
33.

## 2. Users

| User | What they do with archgram |
|---|---|
| A developer documenting a system | Writes or edits a spec next to the code and regenerates the diagram when the system changes |
| An AI coding agent | Reads the code, writes the spec, runs archgram, checks the result; archgram removes the layout work |
| A writer of docs, a design doc or an engineering blog | Draws what a document describes, each part backed by the sentence that states it, and is told when the text no longer says so |
| The reviewer of a pull request | Sees what a change adds to the architecture, removes from it and changes in it, in one drawing |
| The reader of the diagram | Never touches archgram. Everything is judged by what they understand in thirty seconds |

## 3. Product promise

Describe the system, not the drawing. A spec lists typed nodes (a service,
a database, a queue, a model), the frames that group them, the edges
between them and, optionally, the flows through them. archgram lays it out,
routes every edge around the boxes, draws it in one consistent minimalist
visual language, in light and dark, animated when asked, and produces the
same file every time.

## 4. Principles

1. Reader first. Every default serves the person reading the diagram,
   not the person writing the spec.
2. Semantic, not visual. The spec says what a thing is; archgram decides
   how it looks. No coordinates in the spec.
3. Deterministic. The same spec gives a byte-identical file on every
   machine, so diagrams diff cleanly in git.
4. Our own engine. Layout and edge routing are written in-house. Papers
   and open-source engines are read as examples of good practice, not
   copied and not treated as the source of truth.
5. Few, trusted dependencies. The core depends on three crates; every
   other capability is a separate, optional module.
6. Minimalist. Neutral shapes, colour only where it carries meaning.
7. Accessible. Text meets WCAG 2.1 AA contrast in both themes, motion
   stops under reduced motion, every SVG carries a title and a
   description.
8. Themeable. Switching palette is one line in the spec or one flag on
   the command line. A project's own design system is used only when
   asked for.
9. Shared meaning. A kind of diagram other than architecture takes its
   meaning from the standard that defines it: UML for sequence and state
   machine diagrams, BPMN for processes, structured analysis for data
   flow. archgram draws a named subset of it, says what it leaves out and
   claims no conformance; principles 1 to 8 decide how it looks
   ([docs/features/diagram-kinds.md](features/diagram-kinds.md)).

## 5. Scope by version

What each released version brought is in CHANGELOG.md; this table holds
what is next.

| Version | Delivers |
|---|---|
| 0.13 | The C4 notation (a card is its name and its kind; external systems grey and solid; a legend of the kinds drawn); one skill for each kind of diagram, drawing an architecture at the container level from one prompt (§6.6) |
| Planned | Three more kinds of diagram, each from its standard: dataflow, workflow, lifecycle ([docs/features/diagram-kinds.md](features/diagram-kinds.md)) |
| Planned | `archgram diff`: two architecture specs drawn as one diagram, with what the change added, removed and changed marked ([docs/features/diff.md](features/diff.md)) |
| Later | The WASM package, for the browser |

## 6. Functional requirements

### 6.1 Spec
- Input is JSON in the core and YAML through the optional module; both map
  to the same types.
- Unknown fields and unknown node kinds are errors, reported with their
  location, never ignored.
- The spec holds no coordinates. It may carry layout hints: direction,
  which nodes share a column or a row, and the order of nodes within one.
  The direction may be left to archgram (`auto`), which chooses the one
  that fits (§6.2). It may say how wide it is shown (`shownWidth`), a
  README on GitHub when it does not
  ([docs/features/shown-width.md](features/shown-width.md)).
- A node or an edge may name what backs it, the code or the document that
  states it, and archgram says when that is no longer there; it is never
  drawn. It finds what the code or the text lost, not what it gained
  ([docs/features/sources.md](features/sources.md)).
- A spec says which kind of diagram it is, `diagram: sequence`, and
  archgram draws it by that kind's rules; a spec that does not say is an
  architecture diagram, as every spec is today, and draws the same bytes
  ([docs/features/diagram-kinds.md](features/diagram-kinds.md)).

### 6.2 Layout and routing
- Edges flow in one main direction, left to right or top to bottom.
- With the direction `auto`, it is left to right while that keeps the
  text readable where the diagram is shown (no wider than 1,300 px in a
  README, `shownWidth × 1300 / 880` elsewhere, §6.6), and otherwise
  whichever of the two directions is narrower: top to bottom can be wider
  still, for a wide, shallow tree.
  The choice is made from the spec alone, so the same spec still draws
  the same bytes; a spec that names no direction keeps left to right.
- No two boxes overlap. No edge passes through a box it does not start
  or end at.
- Edges are orthogonal, with as few bends and crossings as the layout
  allows.
- Frames contain their nodes, and an edge may cross a frame's border.
- Each kind of diagram lays out by its own rules, all deterministic and
  none from coordinates: architecture, dataflow, workflow and lifecycle in
  layers along the flow, a workflow's nodes each in its lane; a sequence
  in columns, one for each participant, with time running down the page.

### 6.3 Rendering
- Each node kind has its own shape, so a reader recognises it before
  reading its label. The shapes are icons from the free Stroke Rounded
  set of Hugeicons (MIT), a family drawn to one grid and one stroke,
  copied once and drawn in archgram's own stroke; which icon each kind
  takes is in DESIGN.md (Components: Node card).
- The diagram is monochrome: the icon's shape, not a colour, tells a
  node's kind. Colour belongs to the technology logos and to what happens
  on a flow: one colour where a signal passes a card, another where a step
  refuses it (§6.4). A sequence's messages also take a colour by their
  kind, a call, a send, a reply, a refusal, repeating what their shape
  says ([docs/features/sequence.md](features/sequence.md)). A diff's
  marks take a colour too, by what the change did, added, removed or
  changed, repeating what their signs say
  ([docs/features/diff.md](features/diff.md)). A project's
  own tokens may still give each category a hue of its own. Which part
  takes which colour is in DESIGN.md (Colors).
- One SVG carries both themes and follows `prefers-color-scheme`; on
  request archgram writes one file per theme.
- On request, a project's own DTCG design tokens fill archgram's colour
  roles, through a mapping file; the colours must keep the same contrast
  as archgram's own, in both themes.
- Output is self-contained: no script, no external file, no web font.
- Every drawing shows its title above it, "<Kind> diagram of <title>",
  and an optional one-line subtitle under it; the spec hides both
  ([docs/features/approved-drawing.md](features/approved-drawing.md)).
- A small "by archgram" credit sits in the drawing's bottom-right corner,
  faint and hidden from screen readers; the spec turns it off.

### 6.4 Animation
- Off by default. A flow in the spec turns it on for that path, so a
  reader sees in what order things happen. A flow may branch: one step
  may reach several nodes at once.
- A signal travels each step of a flow, drawn in a style the spec picks
  from a fixed set, and a card is lit while the signals it sends are on
  their way, so the reader can follow where the flow is.
- A flow may stop at a step whose node refuses it. The refusal travels
  back to where the flow began, and the refusing card stays marked until
  a later flow passes it.
- An edge's label lights with its signal, and where the still image
  numbers the steps, a step's number lights as its signal reaches it.
- archgram computes the timeline: branches start together, converging
  paths arrive together, the last node of a flow lights last.
- Native SMIL only; every animation stops under
  `prefers-reduced-motion`, and the still image shows what the spec
  chooses: nothing more, the flows in words under the legend, or each
  step's number on its lines. A screen reader hears each flow in words, a
  refused step included.
- In every kind of diagram, what moves is the order the diagram already
  states: a flow's steps, a sequence's messages, a lifecycle's run
  through its states. The rules above hold for each: the timing, a
  refusal, reduced motion, the still image and a screen reader's account.
- How each of these is drawn, in which colours and with what timing, is
  in DESIGN.md (Components: Signal, Refusal; Layout: Motion).

### 6.5 Outputs and interfaces
- A command-line tool, one native binary per platform, installed from npm
  (README.md, Install), so a Node project's scripts run it with no Rust
  toolchain.
- A spec named `<name>.archgram.yaml` (or `.yml`, `.json`) draws `<name>.svg`
  beside it, so a project keeps each spec next to its drawing; a folder
  given for the output is created when it does not exist.
- `archgram spec` prints the spec format this version reads (docs/SPEC.md,
  carried in the binary), so whoever writes a spec, a person or an agent,
  reads the format of the very command that draws it. The format opens
  with a short part, one complete spec that uses each common field and a
  line for each other section, which `archgram spec --brief` prints alone;
  `--section` prints one section. Both are parts of the same document, not
  a copy of it. `--kinds` lists the kinds of diagram this version draws,
  each with the section that holds its format and the question it
  answers, so whoever writes a spec learns from the command which kinds
  it may name.
- `archgram logos <name>...` prints the logo each technology's name
  stands for, its slug as `tech` takes it, or the candidates when it
  stands for several: whoever writes a spec takes each logo from the set
  this archgram draws, rather than guessing a slug (§7).
- `archgram guide` prints the rules the skills share (docs/GUIDE.md,
  carried in the binary), all of them or one section with `--section`,
  so a skill reads the rules of the very archgram it runs (§6.6).
- `archgram build` says the size it drew and the direction, and counts
  what makes its lines hard to follow: lines that cross, lines through a
  frame they neither start nor end in, lines against the main direction,
  and cards reached side by side by lines from different sources. It
  warns, still drawing, when it is wider than keeps its text readable
  where it is shown (§6.2).
- `archgram diff` draws two architecture specs, a change's before and
  after, as one diagram: each part the change added, removed or changed
  carries a mark, in one layout of both, and the command prints the same
  changes as text ([docs/features/diff.md](features/diff.md)).
- A library, usable from Rust and, later through WASM, from the browser
  (§5).
- A still PNG of each theme, `build --preview`, in `.archgram/preview/`
  at the project's top, a folder git ignores by a `.gitignore` of its
  own, rendered inside archgram with resvg from the SVG it draws: for the skill
  to look at the drawing, and for a page that shows no SVG
  ([docs/features/reader-check.md](features/reader-check.md)).
- The skills (§6.6), in the repository's `skills/` folder, for any coding
  agent that reads the open Agent Skills format, Claude Code among them,
  installed with the `skills` command (`npx skills add byabas/archgram`);
  archgram carries no installer of its own. The same skills are one
  Claude Code plugin, `archgram`, listed in Anthropic's directory, and
  the repository is a Claude Code marketplace of that plugin
  (`.claude-plugin/marketplace.json`), so Claude Code installs it where
  no claude.ai account brings it from the directory, and `claude plugin
  update` brings each release.

### 6.6 The skills
- **One skill for each kind of diagram**, named `draw-<kind>-diagram`:
  `draw-architecture-diagram` and `draw-sequence-diagram`, called in
  Claude Code as `/archgram:draw-architecture-diagram`. Each is a folder
  with its own `SKILL.md`, and the next kind a skill of its own. Each
  holds only its kind's rules, so a new kind adds a skill and changes no
  other. The rules every kind shares (what is never read, checking, the
  critique, the report) are one document,
  docs/GUIDE.md, carried in the binary and printed by `archgram guide`
  (§6.5), as the format is by `archgram spec`: written once, kept in no
  skill's folder, and always the rules of the archgram that draws.
- **Small enough to read whole.** Each `SKILL.md` stays under 500 lines
  and about 5,000 tokens, and points to its other files one level deep,
  as the format recommends
  ([best practices](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices));
  a test holds the line count. A rule a command already enforces
  (`check`, `build`) is not repeated in the text: the
  skill runs the command and does what it says.
- **One prompt, one diagram.** The user asks in words, as the README's
  example prompt for each skill shows, or calls the skill by name; the
  skill never guesses another kind from the question. It then draws
  without stopping: no question about the scope, what it found or its own
  files. It asks only before it downloads archgram (below), and stops
  only when it cannot go on: npx or the project's dependencies missing,
  or nothing in the project that the request names. A user who wants to
  see what it found first says so in the request.
- **Drawn from the code.** A part is drawn only when a file in the project
  backs it, an edge only where a line of code makes it, and the spec
  keeps both as sources, so `archgram check` holds the drawing to the
  code now and later ([docs/features/sources.md](features/sources.md)).
  Where the code and a document disagree, the code is drawn and the
  difference is in the report. When the user names a document to draw
  from, the document's sentences are the facts instead, in its own
  words, and what it does not state is not drawn.
- **An architecture diagram is drawn at the C4 container level:** each
  app, service, job, database, queue and outside system one node, the
  lines running from the caller to what it calls, in one direction
  ([C4: container](https://c4model.com/diagrams/container)). The modules
  inside one container (routes, files, functions) are a component
  diagram, drawn only when the user asks for it. The system's style
  (client and API, microservices, a pipeline) says what counts as a
  container in it, never the level.
- **It reads only what the diagram needs,** never the whole codebase, as
  a coding agent understands an architecture: the file list, each app's
  manifest, its run files and entry points, and the line behind each
  line it draws. The spec's sources are the record of what backs the
  drawing; no other document repeats them.
- **A drawing a reader takes in at a glance:** about ten nodes and twelve
  edges, `build`'s counts at 0, no wider than keeps its text readable
  where it is shown (1,300 px in a README, `shownWidth × 1300 / 880`
  elsewhere). Over a limit, it shows less: parts with the same relations
  merged, what the reader does not need left out and said so. It splits
  a drawing in two only when the user asks, and a drawing it replaces,
  it removes. Where the request and a limit disagree, it draws what was
  asked and reports the limit's numbers.
- **It looks before it reports:** it builds with `--preview`, critiques
  the still image of each theme as a reader sees it, and fixes the spec
  until it passes; it never starts a browser
  ([docs/features/reader-check.md](features/reader-check.md)). It then
  opens the drawing with the system's viewer and reports in about
  fifteen lines: each drawing, how exact it is, what it changed after
  looking, what it left out, what archgram could not draw.
- **It runs archgram from where it already is:** the project's own
  when its `package.json` lists one, else the version released with the
  skills, named exactly (`archgram@X.Y.Z`), when the machine has it.
  When the project's own is older than the skills' version, the skill
  says so, with the project's own command to upgrade, and draws nothing:
  an older archgram has neither the guide nor the commands the skills
  run.
  When it finds none, it asks before downloading: what (the package, its
  version, the native program) and from where (the npm registry),
  offering to install it into the project or to let npx fetch it once.
  npx assumes yes when no terminal is attached, so the skill asks
  instead. Anthropic's directory requires the exact version
  ([pre-submission checklist](https://claude.com/docs/plugins/pre-submission-checklist));
  each release sets it, and a test holds the skills to it.
- **It learns the spec format from archgram** (`archgram spec`), so what
  it writes matches the command that draws it; it keeps no copy of the
  format.
- **Colours only when asked:** the project's DTCG tokens, else the colours
  its code styles with, else colours the user gives, written as tokens
  and a mapping beside the spec; archgram's contrast check holds.
- **The skills are text and images alone,** with no script of their own,
  as Anthropic's directory asks of a plugin from an organization it has
  not Verified. The skills are written to the Agent Skills format alone and
  name no one agent's tools. Their evaluations live beside them and run
  on request, since each run is a paid model call.

## 7. Node vocabulary

Every kind has three variants: single, multi-node (several instances,
drawn as stacked boxes) and external (a system we do not own, drawn as
a card tinted grey and edged solid; a dash marks a boundary only). The kinds, in four categories, are in docs/SPEC.md
(Nodes).

A node may name its technology (`tech: postgresql`, a Simple Icons slug);
archgram then shows the technology's logo, so a reader knows the
technology at a glance. A card reads as C4 writes an element: its name,
and its kind under it; with a logo, the name says what the part does
here ("Payments"), since the logo already says which technology it is,
and `check` warns of a name that only repeats it. A node's note is never
drawn, so a card stays two lines. Where the logo goes and in what colour is in
DESIGN.md (Components: Technology logo). Logos come from a pinned Simple
Icons release, every logo it holds, each with its licence. The brands'
guidelines and the logos' licences stay with whoever publishes a diagram,
and archgram records each logo's source, guidelines and licence for them.

Each other kind of diagram has its own vocabulary, from its standard: a
sequence's participants and messages, a lifecycle's states and
pseudostates, a workflow's tasks, events, gateways and lanes, a data
flow's processes, stores and external entities. The kinds above and their
icons are reused where they mean the same thing, a database as a data
store, a person as an external entity. The vocabularies are in
[docs/features/diagram-kinds.md](features/diagram-kinds.md), and in
docs/SPEC.md once built.

## 8. Success criteria

- The two README diagrams of ai-powered-cv-screener are reproduced from
  specs with no coordinates, and a reviewer prefers or accepts them.
- On a test set of specs: no overlapping boxes, no edge through a box,
  byte-identical output across runs and across macOS, Linux and Windows.
- Every text pair passes WCAG 2.1 AA in both themes.
- Layout and SVG for a 100-node spec take under 50 ms in the native CLI.
- The core WASM module, when there is one, stays under 350 KB gzipped.
- A skill's run asks the user nothing on its way to the drawing but
  whether to download archgram, when none is there, and loads no more
  than its own kind's rules and the shared ones.
- With the skill, an agent passes at least 90 % of the evaluation's
  checks (`evals/archgram/evals.json`) and at least 75 % of each case's,
  and more of them than the same agent without it. The evaluation of §1
  passed 64 of 71 with the skill (90 %, its lowest case 69 %) and 31 of
  71 without.
- With the skill, an agent draws a small project for the first time in
  under 45 seconds, and repairs a diagram whose code changed in under 30
  (the cases `draws-a-project` and `repairs-what-lost-its-code`, the
  median of three runs on Claude Opus 5.5). The evaluation of §1 took
  74 and 36.
- Each feature with a document of its own meets the criteria there:
  [what backs a diagram](features/sources.md#success-criteria),
  [where a diagram is shown](features/shown-width.md#success-criteria),
  [more kinds of diagram](features/diagram-kinds.md#success-criteria),
  [what a change does to the architecture](features/diff.md#success-criteria),
  [what was approved, drawn to be followed](features/approved-drawing.md#success-criteria).

## 9. Non-goals

- An interactive editor or any GUI.
- Free-form drawing: a box at a chosen position, a line with a chosen
  path.
- Other diagram types: entity-relationship, class, Gantt. A sequence,
  state machine, process or data flow diagram beyond the subset its
  document names: archgram is not a UML or BPMN modelling tool and claims
  no conformance to either.
- A hand-drawn or sketch style.
- Graphs larger than about 300 nodes.
- A hosted rendering service.
- A layout that changes with the reader's screen. A diagram keeps one
  shape, so a reader's picture of it holds from one screen, and one
  version, to the next ([docs/features/shown-width.md](features/shown-width.md)).

## 10. Open questions

None.

## 11. Changelog

Each draft's changes are in [PRD-history.md](PRD-history.md).
