# The skills' guide

The rules every `draw-<kind>-diagram` skill shares (docs/PRD.md, 6.6).
`archgram guide` prints all of it, `archgram guide --section <name>` one
section. A skill's own files hold only its kind's rules.

## In brief

- **Accuracy first:** nothing is drawn that the code, or a document the
  user named, does not hold, not even when asked. Every box has its file
  and every line its line of code, as the spec's `source`.
- **Then the request:** what to draw, which part, which direction.
- **Then the limits** below, only where the request says nothing; where
  they disagree, draw what was asked and give the numbers in the report.
- **One prompt, one diagram.** Go from the request to the drawing without
  asking. Stop only when you cannot go on, and say why. When the user
  asks to see what you found first, show it before writing the spec.
- **Read only what the diagram needs,** never the whole codebase.
- Each `archgram` command runs as the skill runs archgram. Run each on its
  own, not joined to another with `&&`, `;` or a pipe.

## What is never read

What commonly holds secrets is never opened, not even to look: `.env`,
`.env.*`, `.npmrc`, `.pypirc`, `.netrc`, `.git-credentials`, `*.pem`,
`*.key`, `*.p12`, `*.pfx`, private SSH keys, and anything in `.ssh`,
`.aws`, `.gnupg` or `.docker`. Nor is what is never the project's own:
vendored folders, lock files, minified and generated files, media.

## From a document

When the user names a document to draw from, its sentences are the
facts in place of the code: a box only where a sentence states it, a line
only where a sentence says one part calls, sends to or reads from
another, each sourced to a few words of that sentence
(`docs/design.md#the worker takes each job from the queue`), in the
document's own words. What it does not state is not drawn.

## Check and draw

`archgram check` after every change to the spec, the last one included:
a source the code lacks is a fact you could not back, so drop it or
find where the code went. Fix each warning too. Then `archgram build
<spec> --preview` (with `--theme-file archgram.theme.json` when the
project's root has one); read the still images from the paths it prints.

**The limits:** about ten boxes; `build`'s layout counts at 0; no wider
than 1,300 px where no place is named. Over a limit, show less: merge
parts the reader need not tell apart, leave out what no question needs.
Split a drawing only when the user asks, and remove a drawing a split or
a rename replaces.

## The critique

Look at both previews as the reader will, fix what you find in the spec,
build again, and look again. Never start a browser.

1. Can the reader tell, from the image alone, what the system is made of
   and the main path through it?
2. Is every box and line in the code, named as the code names it?
3. Does every label read, between the parts it joins, as a short
   sentence of what happens?
4. Do arrowheads crowd into one box, or lines run round the drawing?
5. Is every name, label and logo readable in light and in dark?

Then open the drawing once with the system's viewer: `open` on macOS,
`xdg-open` on Linux, `start ""` on Windows.

## The report

About ten lines: each drawing's path and what it shows; that `check`
passes, with its count; where the code and a document disagree; what
changed after looking; what was left out and why, and any limit still
over, with its number. Edit a README, or commit, only when asked.

## Colours

Only when the user asks for the project's colours or names their own:
write `archgram.theme.json` at the project's root (`archgram spec
--section theme-file`), mapping archgram's roles by meaning to the
project's DTCG tokens, else to the colours its styling code holds,
written as tokens at `docs/diagrams/theme/archgram.resolver.json`, else
to the colours the user gave. Then `archgram theme check
archgram.theme.json`; a pair below archgram's contrast is refused: map a
stronger token, or keep archgram's colour for that role and say so.
