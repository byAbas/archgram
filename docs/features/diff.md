# Feature: what a change does to the architecture

| Field   | Value      |
|---------|------------|
| Version | 0.2        |
| Date    | 2026-11-14 |
| Status  | Draft      |
| Release | Planned    |
| Issue   | [#47](https://github.com/byAbas/archgram/issues/47) |

What this feature does and why, in more detail than `docs/PRD.md`, which
points here. The command goes into `README.md` and the marks it draws
into `DESIGN.md` as it is built; how two specs become one drawing goes
into `ARCHITECTURE.md`.

## Problem

A pull request shows the lines that changed, not what changed in the
architecture: a new service, a new call to the database, a queue taken
out. A reviewer rebuilds that picture in their head, and more and more
often for a change an agent wrote.

The spec already holds the architecture before and after, in two
versions of one file. Nothing draws the difference. Drawing each version
with `build` and looking at both does not show it either, because the
layout is computed and a change moves it (Evidence).

### Evidence

- Measured on 2026-11-14 with archgram 0.10.1's layout: five of the
  architecture examples in `examples/` (6 to 18 nodes), each changed 20
  times by taking one or two nodes out, with their edges, and putting as
  many new ones in, 100 pairs in all, flows and hints left out.
  - The old spec's drawing and the new spec's never had every kept card
    in the same place, in 0 of 100 pairs; between 56 % and 76 % of the
    kept cards moved against the others. Two drawings side by side
    cannot be compared at a glance, as
    [where a diagram is shown](shown-width.md#problem) found for one
    edit.
  - Both specs laid out together, in the order the requirements give,
    drew the new spec's own layout in 3 of 100 pairs and kept the cards'
    order in 10; with the removed parts last in the file instead, in 6
    and 9.
  - The drawing of both was 1.17 times as wide as the new spec's on
    average, and 2.2 times at most.
- archify (tt-a1i/archify) has had `compare architecture` since 2.13.0:
  two snapshots in, a Before / Delta / After page and a JSON receipt
  out, parts paired "only by authored stable IDs", each change marked
  with `+ / ~ / − / ↔` and a line pattern (its `CHANGELOG.md`). A removed
  part keeps its place there because its position is written by the
  author, not computed, which archgram's is not.

## Who it serves

| User | What they get |
|---|---|
| The reviewer of a pull request | What the change adds, removes and changes in the architecture, in one drawing and a few lines of text |
| The author, a person or an agent | The same list for the pull request's description, from the two specs, with nothing to write by hand |
| A reader of a design doc or a changelog | A proposal or a release drawn against what is there today |
| The user of the `archgram` skill | After an update, what changed in the diagram as the command found it, not as the agent recalls it |

## Requirements

- `archgram diff <old spec> <new spec> -o <file>.svg` draws one diagram
  of both specs. Each is read and checked as `build` reads it, JSON or
  YAML, and an error names the file it is in.
- Both are architecture specs. Another kind is refused by name.
- Parts are paired by what the spec already makes unique: a node and a
  frame by `id`, an edge by its `from` and `to` (two edges between the
  same pair in the same direction are an error, docs/SPEC.md, Edges).
- A part is added, removed, changed or the same. It is changed when
  something a reader sees of it differs: a node's `kind`, `label`,
  `note`, `tech`, `variant` or `frame`; an edge's `label` or `style`; a
  frame's `label` or `parent`. A `source`, a part's place in the file, a
  flow and a hint are not changes.
- One layout holds both specs (Decided, 1). The parts are the new spec's,
  in its order, with each removed part put back after the part it
  followed in the old spec and into the hints that named it. The
  direction, the palette, the hints and `shownWidth` are the new spec's.
  A mark takes no room from the layout. So a change that only adds draws
  every card where `build` draws the new spec, and one that only removes
  where it drew the old.
- Each added, removed and changed part carries a mark (Decided, 2): a
  sign in a filled circle, `+` for added, `−` for removed, `~` for
  changed. A card carries it on its top edge; a line carries it on a
  straight stretch, clear of cards, labels and bends. The legend names
  each mark the drawing uses.
- A mark has a colour, green for added, red for removed, amber for
  changed: the circle, the card's border, and a line with its arrowhead
  take it. The colour repeats the sign and never stands for it, so a
  mark is told from the others without its colour. Each colour holds 3:1
  against the card and the canvas, in both themes.
- A removed card's title is struck through; its text keeps its colour
  and its contrast. A removed part is not drawn with the dashes that
  already mean a system we do not own, a boundary or a path taken only
  sometimes (DESIGN.md, Overview; docs/SPEC.md, Edges).
- A changed part is drawn as the new spec has it. What it was is in the
  text the command prints and in the SVG's description.
- It is a still image: no flow plays in it.
- The SVG is what `build` writes: self-contained, light and dark in one
  file, every text pair at WCAG 2.1 AA, a title and a description, where
  a screen reader hears each change in words.
- The command prints each change on a line of its own, in the order of
  the drawing's parts, then the size it drew, as `build` does, with the
  same warning when it is wider than keeps its text readable for the new
  spec's `shownWidth`.
- The same two specs give the same bytes and the same text, on every
  machine.
- The `archgram` skill uses it on an update (Decided, 3). It keeps the
  spec as it was before it changes it, and once the new spec passes
  `check` and is drawn, runs `archgram diff` on the two. What changed in
  its report is what the command printed; why each part changed is still
  the skill's to say, starting with what `check` said had lost its code
  (PRD §6.6). A first drawing has nothing to compare with, and a sequence
  is not compared (Not in this).

## Limits

- The drawing is the diff's own. When a change both adds and removes, it
  is neither the old spec's layout nor the new one's (Evidence), so a
  reader who knows the README's diagram finds a part by its mark and its
  name, not by where it was.
- It is wider than the new spec's drawing, since it holds the removed
  parts too.
- It compares two specs, not the code. It is as true as the specs are;
  whether a spec still matches its code is `archgram check`'s question
  ([what backs a diagram](sources.md)).
- A part whose `id` was renamed is one part removed and one added.
- It shows what changed in the parts, not what the change risks or
  reaches.
- The skill's own text changes when the command is built, not before,
  and by review, as the code does (PRD §6.6).

## Not in this

- Diffing code. The input is two specs; the spec stays the source.
- Reading git. The command takes two files; whoever calls it takes the
  old one from the base branch.
- A GitHub Action that draws the diff of each changed spec and posts it
  on the pull request. It follows once the command exists.
- Sequence diagrams and the kinds planned after them.
- What changed in the flows.
- Before and after as panels beside the diff.

## Success criteria

- A spec compared with itself reports no change.
- For a change that only adds, every card of the new spec is where
  `build` draws it; for one that only removes, every card of the old
  spec is where `build` drew it.
- On a test set of pairs: goldens, byte-identical output across runs and
  across macOS, Linux and Windows; no overlapping boxes and no edge
  through a box, removed parts included.
- Every mark and its text pass WCAG 2.1 AA in both themes, and each mark
  is told from the others with the colours taken away.
- On the test set, a reader lists the changes from the still image
  alone, and the list matches the text the command prints.
- In the skill's evaluation, the report of an update lists the changes
  `archgram diff` prints for the spec before and after, no more and no
  fewer (`updates-an-existing-diagram`, `repairs-what-lost-its-code`).

## Decided (2026-11-14)

1. **One layout of both specs together.** The other way, the new spec's
   own layout with the removed parts placed into it, would keep the
   README's picture, but needs a layout that holds some cards still
   while it places others, and a removed part's lines would have to find
   a way through a layout made without them. Before and after side by
   side was not taken either: the two layouts differ too much to compare
   by eye (Evidence).
2. **The marks are signs in colour**, the look `signs` of three samples
   ([docs/samples/diff/](../samples/diff/README.md)). The same signs
   with no colour (`mono`) kept the drawing monochrome, but the changes
   were harder to find at a glance. Words on the cards (`words`) need no
   legend there, but a word does not fit a short line, so the lines
   would carry signs anyway. In the dark theme the three colours are
   nearly as light as one another (1.1 to 1.4 between them), so it is
   the sign that tells them apart for a reader who sees no colour.
3. **The skill uses `diff` on an update.** Today it notes what it
   changed as it goes and reports from those notes. With `diff` the
   report rests on the two specs, and a change an agent wrote, where
   this feature's problem is greatest, gets its list without anyone
   asking. Leaving the skill as it is would have cost nothing, and left
   `diff` to people and scripts alone.

## Open questions

- The green of the light theme: the palette's holds 3.11 against the
  card and 3.00 against the canvas, at the limit for a line. The marks'
  colour roles and their values are set in the tokens as the feature is
  built, and may need a darker green there.
- Where an added and a removed line leave one card, they share a trunk
  (DESIGN.md, Components: Connector), which took the colour of the line
  drawn last in the samples. Whether a marked line keeps a port of its
  own.
- Whether a sign reads where the drawing is shown smaller: its circle is
  16 px, about 11 px in a README that shows a 1,300 px drawing at
  880 px. Not checked.
- How a frame carries its mark. The samples have no added, removed or
  changed frame.
- Where the skill writes the diff's drawing, and whether it stays in the
  project or is only opened and shown.
- What the skill does with an archgram that has no `diff`, a project's
  own older one: the proposal is that it reports from its notes, as
  today.
- Whether the skill, asked about a pull request, compares with the base
  branch's spec rather than the spec as it found it.
- What the command does when nothing differs, and its exit code: whether
  a difference is a failure a script can test for.
- Whether the text is also printed as JSON, for a script or an agent.
- Whether the description's change, a spec's `title` or `description`,
  is reported.
- How the Action puts the drawing on a pull request. GitHub takes an
  SVG as an attachment to a comment, and its documentation names the
  browser and GitHub CLI as the ways to attach one
  (https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/attaching-files);
  whether an Action can do the same is not checked.

## Changelog

| Version | Date       | Change |
|---------|------------|--------|
| 0.1     | 2026-11-14 | First draft, from #47: `archgram diff` draws two architecture specs as one diagram, parts paired by id, each added, removed or changed part marked; one layout of both together, measured on 100 pairs; a still image, with the changes as text; the marks, the exit code and the Action left open. |
| 0.2     | 2026-11-14 | The marks are decided, from three samples in `docs/samples/diff/`: a sign in a filled circle on a card's top edge and on a line, in green, red or amber, repeated by the card's border and the line; a removed card's title struck through. Open: the light theme's green, a trunk shared by an added and a removed line, the sign's size where the drawing is shown smaller, a frame's mark. The `archgram` skill uses `diff` on an update, and its report of what changed is the command's; where it writes the drawing and what it does with an older archgram are open. |
