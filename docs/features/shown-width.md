# Feature: where a diagram is shown

| Field   | Value      |
|---------|------------|
| Version | 0.1        |
| Date    | 2026-10-08 |
| Status  | Draft      |
| Release | 0.9        |
| Issue   | [#114](https://github.com/byAbas/archgram/issues/114) |

What this feature does and why, in more detail than `docs/PRD.md`, which
points here. The field it adds is in `docs/SPEC.md` (Top level); how
archgram measures a drawing against it is in `ARCHITECTURE.md` (Layout,
Direction).

## Problem

archgram keeps a drawing's text readable by one rule: at most 1,300 px
wide, the most a README on GitHub, about 880 px wide, shows with its
smallest text at a readable size (PRD §6.2, §6.6). Shown anywhere
narrower, the same drawing is scaled down further and its text falls
below that size, and nothing says so: `build` measures every drawing
against the README. On a reference site whose topic column is 674 px
wide, seven of ten drawings showed at 53 % to 81 % of their size.

A second problem comes from the first's fix. `direction: auto` chooses
the direction from the drawing's width, so an edit that moves the width
across the limit turns the whole drawing on its side: on the same site,
adding one edge and removing two moved a drawing from 1,231 × 290 (left
to right) to 566 × 702 (top to bottom) and back to 1,265 × 370. Each
version is right on its own, but a reader who knew the last one has to
learn the diagram again, and a pull request's before and after cannot be
compared at a glance.

A layout that follows the reader's screen would make both worse: a
reader keeps a mental map of a diagram, and one that moves loses it
(Misue, Eades, Lai and Sugiyama, "Layout Adjustment and the Mental Map",
Journal of Visual Languages and Computing 6(2), 1995). So the place a
diagram is shown belongs to the spec, and the shape it gets there stays.

## Who it serves

| User | What they get |
|---|---|
| The reader | Text at a readable size wherever the diagram is shown, and a diagram that keeps its shape from one version to the next |
| Whoever keeps the diagram, a person or an agent | One field that says where it is shown; `build` measures against it |
| The reviewer of a pull request | A change to the parts shows as a change to the parts, not as a new layout |

## Requirements

- A spec may say how wide it is shown, in CSS pixels: `shownWidth: 674`.
  Without it, a diagram is shown in a README on GitHub, 880 px.
- The widest drawing that keeps its text readable there is
  `shownWidth × 1300 / 880`: the scale a README applies to a 1,300 px
  drawing today, kept for every place. For a README this is 1,300 px, as
  now; for 674 px it is 996 px.
- `build` warns, and still draws, when a drawing is wider than that,
  naming the width it is shown at and what would bring it within: the
  other direction, written in the spec, or two diagrams.
- `direction: auto` chooses as now, against this width instead of
  1,300 px. The choice is still made from the spec alone, so the same
  spec draws the same bytes.
- The skill writes the direction `build` chose into the spec, in place of
  `auto`, once the first drawing is accepted. Later edits keep it. To
  turn a drawing is a deliberate edit of `direction`, which the skill
  makes only when the user asks or when the drawing no longer fits where
  it is shown, and then says so in its report.
- One spec is shown at one width. A diagram shown in two places takes
  the narrower, so both show one picture.

## Limits

- `shownWidth` is the widest the place shows the diagram. A narrower
  screen, a phone, still scales it down: archgram does not lay out per
  screen.
- The rule is about the smallest text. A drawing within it can still be
  hard to read for other reasons: too many parts, long labels.
- A pinned direction does not move when the diagram grows. `build`'s
  warning is then the only signal, and the skill acts on it.
- Width is never a failure: `build` draws a drawing too wide for where it
  is shown and warns, and `check` does not fail on it. The drawing may be
  what the user wants; the warning says what it costs the reader.

## Not in this

- A layout that changes with the reader's screen.
- Several widths for one spec.
- Wrapping long edge labels to narrow a drawing, a separate change that
  would help at any width.

## Success criteria

- A spec with no `shownWidth` draws the same bytes as before.
- On the reference site, with `shownWidth: 674`, every drawing is at
  most 996 px wide, or `build` warns and the skill splits it.
- In the skill's evaluation, an update to an existing diagram never
  changes its `direction` unless the case asks for it
  (`updates-an-existing-diagram`, `repairs-what-lost-its-code`), and a
  first drawing ends with its direction written in the spec
  (`draws-a-project`).

## Changelog

| Version | Date       | Change |
|---------|------------|--------|
| 0.1     | 2026-10-08 | First draft, from #114: `shownWidth`, a README by default; the rule as the README's ratio, 1,300 to 880; `build` warns and still draws; the skill writes the direction it chose into the spec; no layout per screen. |
