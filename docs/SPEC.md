# The archgram spec

The input format: what a diagram is made of and the rules a spec must
follow. Why the format looks like this, and every other fact about
archgram, is in its repository's other documents; `CONTRIBUTING.md` (Each
fact lives in one document) says which holds what.

A spec describes the system, never the drawing. It has no coordinates, no
sizes and no colours. It names things, groups them and connects them;
archgram decides where they go and how they look.

## In brief

One spec that uses each field most diagrams need, and a line for each
other section. `archgram spec --brief` prints this section alone, and
`archgram spec --section <name>` prints another, by the name below.

```yaml
archgram: 1
title: shop
description: >-
  A customer places an order with the API, which queues it; a worker
  saves it to Postgres and mails a receipt through Resend.
direction: auto
nodes:
  - { id: browser, kind: browser, label: Customer }
  - { id: api, kind: service, label: API, tech: express, frame: app, source: ../../src/api.ts }
  - { id: queue, kind: queue, label: orders, tech: redis, source: ../../src/queue.ts }
  - { id: worker, kind: service, label: Worker, frame: app, source: ../../src/worker.ts }
  - { id: db, kind: database, label: Postgres, tech: postgresql, source: ../../src/db.ts }
  - { id: mail, kind: service, label: Resend, variant: external, source: ../../src/mail.ts }
frames:
  - { id: app, label: Our services }
edges:
  - { from: browser, to: api, label: POST /orders, source: '../../src/api.ts#app.post("/orders"' }
  - { from: api, to: queue, source: "../../src/api.ts#orders.add(" }
  - { from: queue, to: worker, source: "../../src/worker.ts#new Worker(" }
  - { from: worker, to: db, label: insert order, source: "../../src/worker.ts#saveOrder(" }
  - { from: worker, to: mail, label: receipt, style: dashed, source: "../../src/worker.ts#sendReceipt(" }
flows:
  - { name: an order, steps: [browser, api, queue, worker, [db, mail]] }
```

A writer that reads the code, or a document the user names, gives a
`source` to every node a file or a sentence backs and to every edge, as
above, and quotes each source that holds a `#`.

- `formats`: JSON or YAML, and the file name that draws `<name>.svg`.
- `top-level`: every field above the lists, with its values and
  default, `direction` and how a flow is drawn among them.
- `nodes`: a node's fields, and every kind by category.
- `frames`: a frame's fields, and nesting one in another.
- `edges`: an edge's fields, and what an edge means.
- `flows`: steps, branches, a flow that stops, and how archgram times them.
- `hints`: `first`, `last`, `sameLayer` and `order`.
- `sequence`: a sequence diagram, `diagram: sequence`: participants,
  messages, replies and fragments.
- `sources`: what a source may name, and how archgram looks it up.
- `validation`: what makes a spec invalid.
- `theme-file`: drawing in a project's own colours.
- `examples`: complete specs, in JSON and YAML.

## Formats

The core reads JSON. The optional YAML module reads YAML 1.2 and produces
the same spec, so every example below can be written either way; the
command line tells them apart by the file's extension (`.json`, `.yaml`,
`.yml`). Unknown fields are errors, not ignored, so a misspelt field is
caught instead of silently changing the diagram. In YAML, a key written
twice, a second document, a tag outside YAML's core schema, nesting deeper
than 64 levels and aliases that expand to more than 10 000 values are
errors too, and a plain word that reads as a number or a boolean (`title: 2026`)
must be quoted to stay text.

A spec named `<name>.archgram.yaml` (or `.archgram.yml`, `.archgram.json`) draws
`<name>.svg` beside it, so each spec sits next to its drawing and the
drawing keeps a plain name; any other spec draws its own name with `.svg`.
`-o` names another file, and creates its folder when it does not exist.

## Top level

```json
{
  "archgram": 1,
  "title": "linkshort",
  "description": "A redirect reads the cache and queues the click; a worker counts clicks into Postgres.",
  "direction": "right",
  "card": "horizontal",
  "logo": "corner",
  "palette": "mono",
  "legend": true,
  "signal": "wire",
  "still": "none",
  "border": "spark",
  "wait": "solid",
  "glow": true,
  "credit": true,
  "nodes": [],
  "frames": [],
  "edges": [],
  "flows": [],
  "hints": {}
}
```

| Field | Required | Values | Default | Meaning |
|---|---|---|---|---|
| `archgram` | yes | `1` | | The spec format's version |
| `diagram` | no | `architecture`, `sequence` | `architecture` | The kind of diagram. Without it, a spec is an architecture diagram, and every field in this section and the sections up to Sequence is its. A sequence takes the fields in Sequence, below |
| `title` | yes | text | | The diagram's name; the SVG's `<title>` |
| `description` | yes | text | | The whole diagram in prose; the SVG's `<desc>`, read by screen readers |
| `direction` | no | `right`, `down`, `auto` | `right` | The direction of the flow. `auto` lets archgram choose: left to right while it keeps its text readable where the diagram is shown (`shownWidth`), and otherwise whichever of left to right and top to bottom is narrower. It is chosen from the spec alone, so the same spec draws the same bytes, and `archgram build` says which it chose |
| `shownWidth` | no | a number of CSS pixels, 200 to 4,000 | a README on GitHub, 880 | How wide the diagram is shown where it is read. The widest drawing that keeps its text readable there is `shownWidth × 1300 / 880`: 1,300 px in a README, 996 px for 674. `direction: auto` chooses against it, and `archgram build` warns, still drawing, when a drawing is wider; width never fails `check` (docs/features/shown-width.md) |
| `card` | no | `horizontal`, `vertical` | `horizontal` | The card style for every node |
| `logo` | no | `corner`, `inline`, `chip`, `icon` | `corner` | Where technology logos go: the card's corner, before the note (or the technology's name), a chip on the icon, or in place of the icon |
| `palette` | no | a palette name | `mono` | The palette; light and dark are chosen when rendering |
| `legend` | no | `true`, `false` | `true` | Whether to draw the legend, when the diagram has one (DESIGN.md, Components: Legend) |
| `signal` | no | `wire`, `spark`, `arc`, `comet`, `dot`, `pulse`, `current` | `wire` | How a flow's signal is drawn along the lines (DESIGN.md, Components: Signal) |
| `still` | no | `none`, `legend`, `numbers` | `none` | What the still image shows of the flows, where nothing moves: nothing more, each flow in words under the legend, or each step's number on its lines |
| `border` | no | `spark`, `drain`, `ring`, `afterglow` | `spark` | How a lit card's border is drawn from the arrow that reaches it: both ways round with a dot on each growing end, then draining toward the arrow that leaves, once round clockwise, or fading while the card is lit (DESIGN.md, Components: Signal) |
| `wait` | no | `solid`, `pending` | `solid` | How a refused card waits for a later flow to pass it: its refusal border as drawn, or marching round it as dashes (DESIGN.md, Components: Refusal) |
| `glow` | no | `true`, `false` | `true` | Whether a signal glows, faintly, in the styles that have a glow (`wire`, `spark`, `arc`); nothing else glows |
| `credit` | no | `true`, `false` | `true` | Whether to write a small "by archgram" in the drawing's bottom-right corner, hidden from screen readers |
| `nodes` | yes | list | | At least one node |
| `frames`, `edges`, `flows` | no | list | empty | |
| `hints` | no | object | empty | Layout hints, below |

The lists are empty above only to show the shape; complete specs are under
Examples.

## Nodes

```json
{ "id": "api", "kind": "service", "label": "API", "note": "3 replicas",
  "tech": "fastapi", "variant": "multi", "frame": "vpc" }
```

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | Lowercase letters, digits and hyphens; unique among nodes and frames |
| `kind` | yes | One of the kinds below; decides the icon and the category |
| `label` | yes | The card's title |
| `note` | no | The card's subtitle: one short line |
| `tech` | no | A technology, by its Simple Icons slug (`postgresql`, `redis`, `react`); shows its logo. Lowercase letters, digits and `_` |
| `variant` | no | `single` (default), `multi` (several instances), `external` (not ours) |
| `frame` | no | The id of the frame the node sits in |
| `source` | no | What backs the node, its code or the document that states it: a path, or a list of paths for a node that stands for several parts (Sources, below) |

Kinds, by category:

| Category | Kinds |
|---|---|
| Core | `service`, `database`, `queue`, `cache`, `storage`, `users` |
| AI and LLM | `model`, `vector-store`, `tool`, `agent` |
| Build and tooling | `file`, `script`, `generated`, `check` |
| Clients | `browser`, `mobile`, `desktop` |

## Frames

```json
{ "id": "vpc", "label": "VPC" }
{ "id": "private", "label": "Private subnet", "parent": "vpc" }
```

A frame is a boundary around nodes: a network, a trust zone, a team's
service. `parent` nests one frame in another. A frame with no nodes and no
child frames is an error.

## Edges

```json
{ "from": "api", "to": "redis" }
{ "from": "api", "to": "postgres", "label": "on a miss", "style": "dashed" }
```

| Field | Required | Meaning |
|---|---|---|
| `from`, `to` | yes | Node ids; the edge points from `from` to `to` |
| `label` | no | A few words on the line |
| `style` | no | `solid` (default) or `dashed`, for a path taken only sometimes |
| `source` | no | The code that makes the edge, usually a path with the words of its line after `#` (Sources, below) |

An edge means data or a call moving in the direction of the arrow. Two
edges between the same pair in the same direction are an error; use one
with a label.

## Flows

```json
[
  { "name": "a visit", "steps": ["browser", "api", "redis", "worker", "postgres"] },
  { "name": "a question", "steps": ["chat", "llm", ["find", "count"], "index"] },
  { "name": "a commit with a raw colour", "steps": ["person", "precommit", "core", "checks"], "stop": "checks" }
]
```

A flow is a path through the diagram that archgram animates, in order. A
step is a node id, or a list of ids reached at once: a branch. Every node
of a step must be reached by an edge from a node of the step before it,
and when a step has several nodes, each must have an edge to a node of
the step after it. A spec without flows gives a still diagram. When
several flows exist they play one after another, in the order listed.

A flow may stop: `stop` names the node that refuses it, which must be
the flow's last step and a single node. A ✕ marks the line into it, and
the refusal travels back along the flow to the node where it began
(DESIGN.md, Components: Refusal). The refusing node stays marked until a
later flow passes it, so a refused flow followed by one that passes the
same node shows a retry.

archgram times the flows itself (DESIGN.md, Layout: Motion): a signal's
speed is fixed, so a longer line takes longer; a branch's signals leave
together; signals that meet at a node arrive together. A card is lit
from the moment a signal reaches its arrowhead until every signal it
sends has arrived; the last card of a flow stays lit until its border
has closed. A refused flow's refusal travels back before the next flow
starts. A screen reader hears each flow in words, after the description,
a refused flow ending with the node that refused it.

## Hints

The layout needs no help, but it accepts some.

```json
{
  "hints": {
    "first": ["browser"],
    "last": ["postgres"],
    "sameLayer": [["worker", "redis"]],
    "order": [["api", "worker"]]
  }
}
```

| Hint | Meaning |
|---|---|
| `first`, `last` | These nodes go in the first or last layer |
| `sameLayer` | Each list shares one layer (one column when the flow runs right) |
| `order` | Within a layer, each list keeps this order, top to bottom or left to right |

A hint that contradicts the edges (a node placed before the node that
feeds it) is an error, reported with both nodes named.

## Sequence

A sequence diagram shows, in order, the messages between participants:
who calls whom, what comes back, and where a request is turned away. Its
meaning is UML's (Unified Modeling Language 2.5.1, clause 17); archgram
draws the part of it listed here (docs/features/sequence.md).

```yaml
archgram: 1
diagram: sequence
title: sign in
description: >-
  The browser posts the login to the API, which finds the user in
  Postgres; with the right password the API returns a session, and
  otherwise refuses with a 401.
participants:
  - { id: browser, kind: browser, label: Browser }
  - { id: api, kind: service, label: API, tech: nodedotjs, source: ../../src/api.ts }
  - { id: db, kind: database, label: Users, tech: postgresql }
messages:
  - { from: browser, to: api, label: POST /login, source: "../../src/api.ts#app.post(\"/login\"" }
  - { from: api, to: db, label: find user, source: "../../src/api.ts#db.users.find(" }
  - { reply: db, label: user }
  - alt:
      - when: password matches
        messages:
          - { reply: api, label: session }
      - when: else
        messages:
          - { reply: api, label: "401", refused: true }
```

### Top level

| Field | Required | Values | Default | Meaning |
|---|---|---|---|---|
| `archgram`, `title`, `description` | yes | | | As in Top level |
| `diagram` | yes | `sequence` | | This section's kind |
| `participants` | yes | list | | At least two |
| `messages` | yes | list | | At least one; their order is time's |
| `still` | no | `numbers`, `none` | `numbers` | Where nothing moves, each message's number by its line, or nothing more: in a sequence the order is the meaning |
| `look` | no | `cards`, `avatars` | `cards` | How it is drawn: `cards`, each head a card and a bar on a lifeline while its participant answers a call; `avatars`, each head a circle and no bars (How a sequence is drawn) |
| `shownWidth`, `card`, `logo`, `palette`, `signal`, `border`, `wait`, `glow`, `credit` | no | | | As in Top level |

`nodes`, `frames`, `edges`, `flows`, `hints`, `direction` and `legend`
are errors in a sequence: time runs down the page, and the participants'
order is the list's.

### Participants

```yaml
- { id: api, kind: service, label: API, tech: nodedotjs, source: ../../src/api.ts }
```

A participant has a node's fields, `id`, `kind`, `label`, `note`, `tech`,
`variant` and `source`, with the same values (Nodes). Participants are
drawn left to right in the order listed, each at the head of its own
column.

### Messages

```yaml
- { from: browser, to: api, label: POST /login }
- { from: api, to: queue, label: enqueue, async: true }
- { from: api, to: api, label: hash password }
- { reply: db, label: user }
```

| Field | Required | Meaning |
|---|---|---|
| `from`, `to` | yes | Participant ids; the message goes from `from` to `to`, the same one for a message to itself |
| `label` | no | A few words on the line: the call, or the data |
| `async` | no | `true` for a send that does not wait for a reply; a call waits by default |
| `refused` | no | `true` when the message turns the request away (below) |
| `source` | no | The code that sends the message, as an edge's (Sources) |

A reply has `reply` in place of `from`:

| Field | Required | Meaning |
|---|---|---|
| `reply` | yes | The participant that replies |
| `to` | no | Who it replies to: by default the caller of the latest call to `reply` not yet answered, and when given it must be that caller |
| `label`, `refused`, `source` | no | As a message's |

A call is drawn as a solid line with a filled arrowhead, a send that does
not wait with an open one, and a reply as a dashed line, each with its
label in a pill on the line.

### Fragments

A fragment is an item of a `messages` list that holds messages of its
own, so fragments nest.

```yaml
- alt:
    - { when: cached, messages: [ { reply: cache, label: page } ] }
    - { when: else, messages: [ { from: cache, to: db, label: read } ] }
- opt:
    when: the user asked for a receipt
    messages:
      - { from: api, to: mail, label: send receipt, async: true }
- loop:
    when: for each line
    messages:
      - { from: worker, to: db, label: insert }
- par:
    - { messages: [ { from: api, to: search, label: query } ] }
    - { messages: [ { from: api, to: ads, label: query } ] }
```

| Fragment | Holds | Meaning (UML 2.5.1, 17.12.15) |
|---|---|---|
| `alt` | two operands or more, each with `when` and `messages`; `when: else` only on the last | One of them happens, the first whose guard holds |
| `opt` | one operand, `when` optional, and `messages` | It happens or nothing does |
| `loop` | one operand, `when` optional, and `messages` | It is repeated |
| `par` | two operands or more, each with `messages` | They happen in any interleaving, each in its own order |

An operand may also name the code that makes its choice, such as the
`if` its guard stands for, in `source` (Sources).

A guard (`when`) is a few words. A fragment is drawn as a frame round
the messages it holds, its operator and its first guard in a pill on its
top edge, `alt · cached`; its operands are divided by dashed lines, each
later guard in a pill on its line, `else`.

### Refused messages

`refused: true` marks the message that turns the request away, usually a
reply such as a 401. A ✕ marks its line at the participant that refuses,
and, where the diagram moves, the refusal travels back along the calls
not yet answered to where the request began, as a refused flow does
(Flows). UML has no refusal; it is archgram's.

### How a sequence is drawn

`archgram build` draws a sequence top to bottom, as time runs: the heads
in a row, each participant's column a band down from its head, which is
its lifeline, one row for each message in the list's order, and each
fragment framed round its messages.

`look` picks one of two drawings of the same columns and rows:

| `look` | Heads | While a participant answers a call | Fragments |
|---|---|---|---|
| `cards` | archgram's cards | A bar on its lifeline, from the call to its reply (UML's execution specification) | A solid frame, its pill at the left |
| `avatars` | Circles, the logo in a badge on the edge | Nothing | A dashed frame, its pill in the middle |

A call never answered keeps its bar to the last message its receiver
sends before its caller calls it again, or half a row for a call to
itself; a send, and a refused call, start none; a call to a participant whose
bar is already drawn, a call to itself among them, draws a second bar
over the first, moved to its right.

This version
draws it still, each message numbered by its order (`still: numbers`); a
screen reader hears each message in words, in order, a guard before its
messages. Moving the messages in turn, as flows move, comes in a later
version.

### Sources in a sequence

A participant names the file behind it, and a message the file with a
few words from the line that sends it, as nodes and edges do (Sources).
A reply's and a guard's source is optional. `archgram check` holds them
to the code.

## Sources

A node or an edge may name what backs it, its code or the document that
states it, so archgram can say when that is gone. Sources are never drawn: a spec gives the same SVG
with or without them. Why, and what the check cannot find, is in
`docs/features/sources.md`.

```yaml
nodes:
  - { id: worker, kind: service, label: Worker, source: ../../src/worker.rs }
  - { id: billing, kind: service, label: Billing, source: [../../src/charge.ts, ../../src/refund.ts] }
edges:
  - { from: worker, to: links, source: "../../src/worker.rs#links.insert(" }
```

A source is a path to a file or a folder, from the folder the spec is in,
with its folders separated by `/` on every system and each name with the
capitals it has on disk; it is never absolute, never only `.` and `..`,
has `..` only at its start, and never names `.git` or a file that
commonly holds secrets rather than code (`.env`, `.env.*`, `.npmrc`,
`.pypirc`, `.netrc`, `.git-credentials`, `*.pem`, `*.key`, `*.p12`,
`*.pfx`, `id_rsa`, `id_dsa`, `id_ecdsa`, `id_ed25519`, or anything in
`.ssh`, `.aws`, `.gnupg` or `.docker`). After the first `#` (so a path
cannot hold one), it may carry a few words copied from one line of that
file: the line that makes the edge, or that defines the part, or in a
document the line of the sentence that states it, at least 3 characters
other than spaces. The words are looked for as written, anywhere in the
file, so the source holds while the file changes around them; they are
found in a comment too, so pick words from the code itself. In a
document, take them from within one line: a paragraph wrapped across
lines splits a sentence, and words across the break are not found. `source` is
one source or a list of them, never an empty list, and a spec names at
most 1000 sources.

`archgram check` reports, as problems, each path with nothing there,
each file that does not hold its source's words, and each folder given
words to look for; `archgram build` warns of the same and still draws.
Sources are looked up only under the project's folder: the nearest
folder above the spec that holds `.git`, else the folder archgram runs
in, and never a disk's root. A path that leads outside it is refused by
its text alone, before anything on the disk is looked at. A symbolic
link on the way is not followed. A file is read only to look for a
source's words, once however many sources name it, and only a regular
file of at most 4 MiB; nothing of it is printed or kept. A spec without
sources looks nothing up (SECURITY.md, What archgram reads).

A writer that reads the code, a person or an agent such as the
`archgram` skill, gives a source to every node a file backs (not to the
people or systems outside the code, such as a browser) and to every
edge: the file behind a node (the files, for a node that stands for
several parts), and for an edge the file with a few words copied exactly
from the line that makes it. From `docs/diagrams/`, the paths start with
`../../`. A writer drawing from a document the user names gives every
node and edge the document, with a few words of the sentence that
states it:

```yaml
nodes:
  - { id: queue, kind: queue, label: job queue, source: "../design.md#jobs wait in the job queue" }
edges:
  - { from: queue, to: worker, source: "../design.md#the worker takes each job from the queue" }
```

In YAML, quote every source that holds a `#`: unquoted, ` #` starts a
comment, which drops the rest of the words, and `: ` starts a key.

## Validation

A spec is rejected, with every problem listed at once and each located by
its JSON pointer (or its line and column in YAML), when:

- a required field is missing, a field is unknown, or a value is not one
  of those allowed;
- `shownWidth` is not a number from 200 to 4,000;
- a text (the title, the description, a label, a note or a flow's name)
  holds a character XML does not allow, such as a control character
  other than tab, line feed and carriage return: the SVG would not open;
- an id is duplicated, or an edge, flow, frame or hint names an id that
  does not exist;
- frames nest in a cycle, or a frame is empty;
- the nodes of an `order` hint do not share a frame (a frame keeps its
  nodes together, so a hint cannot sort them among others);
- a hint contradicts the edges: a `sameLayer` group whose nodes an edge
  joins, or a path of edges leads from one to another (through other
  groups too), a `first` node an edge leads into, or a `last` node an
  edge leads out of;
- a flow step is not reached by an edge from the step before it, a node
  of a branching step has no edge to the step after it, a step lists no
  nodes, or a step lists a node twice;
- a flow's `stop` names no node, or a node that is not its last step, or
  its last step is a branch;
- `tech` names a logo archgram does not carry (the error suggests the
  nearest slugs);
- a `source` is empty, absolute, only `.` and `..`, has a `..` after a
  name, names `.git` or a file that commonly holds secrets (Sources),
  holds `\`, has fewer than 3 characters after its `#` or more than one
  line there, or is an empty list; or the spec names more than 1000
  sources;
- `diagram` is not one of those allowed, or a sequence holds a field that
  is not a sequence's (Sequence, Top level);
- a sequence has fewer than two participants or no message, or a
  participant is in no message;
- a sequence's item is not exactly one of a message (`from` and `to`), a
  reply (`reply`) and a fragment, or a fragment carries a message's
  fields;
- a message, or a reply's `reply` or `to`, names a participant that does
  not exist;
- a reply has no call to its participant before it still waiting, or
  names a `to` other than that call's caller; a send that does not wait,
  and a refused call, wait for no reply;
- an `alt` or a `par` has fewer than two operands, an `alt` operand has no
  `when`, `else` is on an operand other than an `alt`'s last, a `par`
  operand has a `when`, a guard is empty, or an operand holds no messages.

`archgram check` then holds each source to the code or the document behind it (Sources).

## Theme file

A project with its own design tokens can draw in its own colours: a
mapping file, `archgram.theme.json` by convention, passed with
`--theme-file` (and checked with `archgram theme check`), names the
project's resolver and which token fills each of archgram's colour roles.

```json
{
  "version": 1,
  "resolver": "design-system/tokens/design.resolver.json",
  "roles": {
    "canvas": "color.surface.page",
    "card": "color.surface.raised",
    "text": "color.text.primary",
    "icon-ai": "{color.accent.violet}"
  },
  "themes": {
    "light": { "inputs": { "theme": "light" } },
    "dark": { "inputs": { "theme": "dark" }, "roles": { "card-edge": "color.border.strong" } }
  }
}
```

| Field | Required | Meaning |
|---|---|---|
| `version` | yes | The mapping format's version. Only `1` exists |
| `resolver` | yes | The project's resolver document (Design Tokens 2025.10), relative to this file, in its folder or below |
| `roles` | no | Each archgram role's token, by its dotted id, with or without braces |
| `themes.light`, `themes.dark` | yes | `inputs`: the resolver's input for each modifier, to pick that theme; `roles`: roles this theme maps differently |

The roles are `badge`, `canvas`, `card`, `card-edge`, `connector`,
`frame`, `icon-ai`, `icon-build`, `icon-client`, `icon-core`,
`signal-core`, `signal-pass`, `signal-refusal`, `text` and `text-muted`
(DESIGN.md, Colors). A role the file
leaves out keeps the mono palette's colour. The file replaces the spec's
`palette`.

archgram reads the resolver's sets, modifiers (an input, else the
modifier's default), its resolution order (later sources win), `$ref` to
files beside it and to its own sets, inline tokens, `{alias}` values and
`$ref` JSON Pointers inside values, and `$type` from the groups above a
token. A colour may be in `srgb`, `srgb-linear`, `hsl`, `hwb`, `oklab`,
`oklch` or `display-p3`; in another space it needs a `hex` to fall back
on. Channels outside sRGB are clipped. A role's colour is opaque.

An import is refused, with every problem listed and each located by file
and JSON pointer, when the mapping or the resolver breaks its format; a
role is not archgram's; an input names no modifier or no context; a
modifier without a default gets no input; a file cannot be read, is a
remote reference, or is outside the mapping file's folder (by its real
path, so neither `..` nor a symlink leads out); a token does not exist (the nearest are suggested), is
not a colour, or aliases in a loop; a group uses `$extends`; a colour
cannot be read; or a pair of roles falls below the contrast DESIGN.md
requires, in either theme.

## Examples

### linkshort (JSON)

```json
{
  "archgram": 1,
  "title": "linkshort",
  "description": "The API creates short links and redirects. A redirect reads the URL from the Redis cache, falls back to Postgres on a miss, adds the click to a Redis stream and returns. A worker drains the stream in batches and adds the counts to Postgres, where the stats route reads them.",
  "nodes": [
    { "id": "browser", "kind": "browser", "label": "Visitor" },
    { "id": "api", "kind": "service", "label": "API", "note": "FastAPI, port 8000", "tech": "fastapi" },
    { "id": "cache", "kind": "cache", "label": "URL cache", "note": "1-day TTL", "tech": "redis", "frame": "redis" },
    { "id": "clicks", "kind": "queue", "label": "Click stream", "tech": "redis", "frame": "redis" },
    { "id": "worker", "kind": "service", "label": "Worker", "note": "up to 500 per read" },
    { "id": "postgres", "kind": "database", "label": "links", "note": "source of truth", "tech": "postgresql" }
  ],
  "frames": [ { "id": "redis", "label": "Redis" } ],
  "edges": [
    { "from": "browser", "to": "api" },
    { "from": "api", "to": "cache" },
    { "from": "api", "to": "postgres", "label": "on a miss", "style": "dashed" },
    { "from": "api", "to": "clicks" },
    { "from": "clicks", "to": "worker" },
    { "from": "worker", "to": "postgres" }
  ],
  "flows": [ { "name": "a visit", "steps": ["browser", "api", "clicks", "worker", "postgres"] } ]
}
```

### linkshort (YAML, the same spec)

```yaml
archgram: 1
title: linkshort
description: >-
  The API creates short links and redirects. A redirect reads the URL from
  the Redis cache, falls back to Postgres on a miss, adds the click to a
  Redis stream and returns. A worker drains the stream in batches and adds
  the counts to Postgres, where the stats route reads them.
nodes:
  - { id: browser, kind: browser, label: Visitor }
  - { id: api, kind: service, label: API, note: "FastAPI, port 8000", tech: fastapi }
  - { id: cache, kind: cache, label: URL cache, note: 1-day TTL, tech: redis, frame: redis }
  - { id: clicks, kind: queue, label: Click stream, tech: redis, frame: redis }
  - { id: worker, kind: service, label: Worker, note: up to 500 per read }
  - { id: postgres, kind: database, label: links, note: source of truth, tech: postgresql }
frames:
  - { id: redis, label: Redis }
edges:
  - { from: browser, to: api }
  - { from: api, to: cache }
  - { from: api, to: postgres, label: on a miss, style: dashed }
  - { from: api, to: clicks }
  - { from: clicks, to: worker }
  - { from: worker, to: postgres }
flows:
  - { name: a visit, steps: [browser, api, clicks, worker, postgres] }
```

### An agentic RAG pipeline (frames, AI kinds, external variants)

```json
{
  "archgram": 1,
  "title": "How an answer comes to be",
  "description": "Before use, the index script has a model extract a profile from each CV, checks every field against the text and embeds each chunk. At question time the chat sends the question to the ask route; the model calls tools over the index and the vectors, ends with present, and the app checks every candidate and page before the answer streams back.",
  "direction": "right",
  "nodes": [
    { "id": "cvs", "kind": "file", "label": "CV PDFs", "note": "30 files", "frame": "prep" },
    { "id": "extract", "kind": "model", "label": "Gemini", "note": "extracts profile", "tech": "googlegemini", "variant": "external", "frame": "prep" },
    { "id": "fieldcheck", "kind": "check", "label": "Field check", "note": "page per fact", "frame": "prep" },
    { "id": "embed", "kind": "model", "label": "Embedding", "note": "each chunk", "variant": "external", "frame": "prep" },
    { "id": "index", "kind": "storage", "label": "data/index", "note": "JSON, in memory" },
    { "id": "vectors", "kind": "vector-store", "label": "Pinecone", "note": "chunk vectors", "variant": "external" },
    { "id": "chat", "kind": "browser", "label": "Chat screen", "frame": "ask" },
    { "id": "route", "kind": "service", "label": "POST /api/ask", "note": "no CV text in the prompt", "frame": "ask" },
    { "id": "llm", "kind": "model", "label": "Gemini", "note": "tool-calling loop", "tech": "googlegemini", "variant": "external", "frame": "ask" },
    { "id": "tools", "kind": "tool", "label": "Exact tools", "note": "find, count, get", "frame": "ask" },
    { "id": "search", "kind": "tool", "label": "search_cv_text", "note": "BM25 + vectors", "frame": "ask" },
    { "id": "answercheck", "kind": "check", "label": "Answer check", "note": "ids and pages", "frame": "ask" }
  ],
  "frames": [
    { "id": "prep", "label": "Before use" },
    { "id": "ask", "label": "At question time" }
  ],
  "edges": [
    { "from": "cvs", "to": "extract" },
    { "from": "extract", "to": "fieldcheck" },
    { "from": "fieldcheck", "to": "index" },
    { "from": "cvs", "to": "embed" },
    { "from": "embed", "to": "vectors" },
    { "from": "chat", "to": "route" },
    { "from": "route", "to": "llm" },
    { "from": "llm", "to": "tools" },
    { "from": "llm", "to": "search" },
    { "from": "tools", "to": "index" },
    { "from": "search", "to": "vectors" },
    { "from": "llm", "to": "answercheck", "label": "present" },
    { "from": "answercheck", "to": "chat", "label": "streams" }
  ],
  "flows": [
    { "name": "a question", "steps": ["chat", "route", "llm", "answercheck", "chat"] }
  ]
}
```
