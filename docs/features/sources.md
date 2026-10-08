# Feature: what backs a diagram, its code or its document

| Field   | Value      |
|---------|------------|
| Version | 0.2        |
| Date    | 2026-10-08 |
| Status  | Draft      |
| Release | 0.7        |
| Issue   | [#46](https://github.com/byAbas/archgram/issues/46), [#122](https://github.com/byAbas/archgram/issues/122) |

What this feature does and why, in more detail than `docs/PRD.md`, which
points here. The field it adds is in `docs/SPEC.md` (Sources); how
archgram checks it is in `ARCHITECTURE.md` (Parse and validate).

## Problem

A diagram kept as text beside the code is cheap to change, but nothing
says when it must change (PRD §1). The `archgram` skill already finds the
file behind each part and the line behind each edge (PRD §6.6), but it
only lists them in its report: the spec keeps none of it, so nothing can
tell later that a part or a line has lost its code. With agents changing
a project many times a day, the diagram is wrong before anyone looks.

A diagram drawn from a document rather than from code has the same
problem: a pattern on a docs site, a system in a design doc or an
engineering blog post. On a reference site, a frontend system design
reference, ten topic diagrams were redrawn from each topic's prose, and
four of the Mermaid diagrams they replaced contradicted their own text.
Nothing had said so: a diagram drawn from prose kept no tie to it.

## Who it serves

| User | What they get |
|---|---|
| Whoever keeps the diagram, a person or an agent | Told what lost its code, at the spec's line, whenever they check or draw |
| The reviewer of a pull request | The `archgram check` CI already runs fails when a change removes code a diagram points at |
| The skill | What it claims about the code is checked as it draws, so a part or a line it cannot back is caught before the drawing |
| A writer of docs, a design doc or a blog | Each part and line tied to the sentence that states it, and told when the text no longer says so |

The reader of the diagram never sees a source. They get a drawing that
stays true.

## Requirements

- A node and an edge may each name what backs them, `source`: the code,
  or the document that states them. It is a path, or a list of paths for
  a node that stands for several parts. A path names a file or a folder,
  from the spec's folder.
- A source may carry, after `#`, a few words copied from the line that
  makes it, such as `../../src/worker.rs#links.insert(`. The words are
  looked for as written, so a source holds through edits elsewhere in the
  file and fails only when that code itself changes or goes. A line
  number would fail at every edit above it, and still say nothing about
  the call. For a document, the words come from the sentence that states
  the part or the line, from within one line of the file.
- Sources are never drawn: the SVG is the same with or without them.
- The check runs where archgram already runs, with nothing to turn on:
  - `archgram check` reports each source the code lacks as a problem, at
    its line and column, and fails;
  - `archgram build` still draws, and warns of each one;
  - the skill writes a source for every node a file backs and every
    edge, so its `archgram check` holds them to the code as it draws; on
    a later call, what the check names is what the skill updates;
  - drawing from a document, which it does only when the user names one,
    the skill draws a part only where a sentence states it and an edge
    only where a sentence says one part calls, sends to or reads from
    another, writes that sentence's words as the source of each node and
    edge (100 %), labels them in the document's own words, and reports
    what the document does not state as a gap.
- How to write a source is in the format (`docs/SPEC.md`, Sources), which
  `archgram spec` prints, not in the skill. The skill is installed apart
  from archgram and may be newer or older than the archgram it runs; from
  the format, it writes sources exactly when that archgram reads them,
  with no version check in the skill and no change to it for this field.
- It reads files only: no git, no network, no clock, so CI can run it on
  every pull request and the same files always give the same answer, on
  every system (a name must have its capitals on disk).
- The check answers alike from any folder of the project: sources are
  looked up under the project's folder, the nearest above the spec that
  holds `.git`.
- A spec may come from someone else, such as a pull request, so a source
  is looked up only under the project's folder, through no symbolic
  link, never `.git` or a file that commonly holds secrets, and a file
  is read only for a source's words, once, at most 4 MiB of it
  (SECURITY.md, What archgram reads). The words are the spec author's
  own: the check says whether a file under the project holds them, and
  prints nothing of it.

## Limit

The check finds what the code lost, not what it gained: a new part or a
new call is in no spec, so nothing points at it. Finding those is the
skill's work when it runs again.

It finds words wherever they are in the file, in a comment or dead code
too, and cannot tell what the code does with them; only a parser of each
language could, and archgram keeps to text. A folder passes while it
exists, whatever it holds.

In a document the same holds for prose:

- the words must sit on one line of the file, and a paragraph wrapped
  across lines splits a sentence, so the skill quotes from within a line;
- a phrase the document repeats elsewhere can hide a change;
- a rewording that keeps the meaning fails the check, and the skill
  updates the quote; a rewording that changes the meaning but keeps the
  quoted words passes.

## Not in this

- Redrawing by itself. The check says what lost its code; the skill, or a
  person, changes the spec.
- Checking what the code does: that the line still writes to the table
  the edge points at.

## Success criteria

- In each spec the skill writes, every node a file backs and every edge
  carries a source (100 %), and `archgram check` confirms each one when
  the drawing is made.
- On the skill's evaluation projects, after a change that deletes a part
  or the call behind a line, `archgram check` names each one affected,
  and after a change that keeps them, names none
  (`evals/archgram`, `repairs-what-lost-its-code`).
- Drawing from a document, on six articles of different shapes (two
  Azure Architecture Center patterns, Gateway Routing and Backends for
  Frontends; Martin Fowler's Strangler Fig Application; Sam Newman's
  Backends For Frontends; Discord's How Discord Stores Trillions of
  Messages; a Netflix Tech Blog post), against a list of parts and
  relations a person writes from each first:
  - no part or relation the article does not state;
  - at least 90 % of the listed parts drawn, or reported as left out on
    purpose;
  - every node and edge carries a source, `archgram check` passes, and
    it fails once a sentence that states a part is deleted.
- An evaluation case drawn from Azure's Backends for Frontends (CC BY 4.0,
  with its attribution) passes at the evaluation's usual bar (PRD §8).
  Only an article whose licence allows it is kept in the repository.

## Changelog

| Version | Date       | Change |
|---------|------------|--------|
| 0.1     | 2026-10-02 | First draft, from #46: a source on a node and an edge, a path with the words of its line after `#`; checked by `check` (fails) and `build` (warns), never drawn; the skill writes them and starts an update from what the check names; the limit, and the criteria. |
| 0.2     | 2026-10-08 | From #122: a source may be the document that states a part or a line, not only its code; the skill draws from a document only when the user names one, in the document's own words; the limits of prose; criteria on six articles and an evaluation case. |
