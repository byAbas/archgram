---
name: archgram
description: Draws a project's software architecture as an animated SVG with archgram, for its README and docs, from the code and documentation, or from a document the user names (a docs page, a design doc, an article). Writes a spec in docs/diagrams/, checks and draws it with the archgram command, in the project's own colours when asked, then opens the drawing and lists each part with the file behind it. Also draws a sequence diagram, the order in which the parts call each other for one request, and what comes back, such as a sign-in or a checkout, in phases. Use it whenever the user wants an architecture, system, data-flow, pipeline, sequence or "how it works" diagram, the steps of a request, a diagram for a README, or an existing archgram diagram updated after the code changed or `archgram check` says its code is gone, even if they only say "draw how this works".
license: MIT
compatibility: Requires Node 22 or later, with npx.
---

# Drawing architecture with archgram

archgram lays out the boxes, routes the lines, animates the flows and
draws light and dark in one SVG. Your work is what it cannot do: decide
what the reader must learn, find the facts in the code, and write them as
a spec. If the user said what to draw (a flow, a part of the system, a
direction), draw that.

Every command below runs archgram through npx, so it needs Node 22 or
later. It names `archgram@0.11.0`, the version released with this skill,
never the latest. When the project's `package.json` lists archgram, run
the project's own instead: write each command with `archgram` where it
says `archgram@0.11.0`, and the project's lockfile decides the version.
Keep `--loglevel=error` either way: it keeps npm's own warnings, such as
those about a pnpm project's `.npmrc`, out of archgram's output, and
changes nothing else. If `npx` is missing, or cannot find that version on
npm, say so and stop.

Run each command on its own, as it is written here: not joined to another
with `&&`, `;` or a pipe, and not inside a loop. An agent may approve the
exact commands this skill names, and a joined one asks the user or is
refused. Read files by reading them, one at a time, and change the spec
by editing it, not through the shell.

```
Progress:
- [ ] The kind of diagram
- [ ] 1. Reader contract
- [ ] 2. Facts, each with its file
- [ ] 3. The spec format, from archgram itself
- [ ] 4. Spec written and checked
- [ ] 5. The project's colours, when asked
- [ ] 6. Drawn, critiqued, opened
- [ ] 7. Reported
```

## Choose the kind

archgram says which kinds of diagram it draws:

```bash
npx --yes --loglevel=error archgram@0.11.0 spec --kinds
```

Each line is one kind: the value a spec's `diagram` takes, the section of
the format that holds its fields, and the question it answers. Draw the
kind whose question the reader asks. "How it works", "the architecture",
a data pipeline, a diagram for a README: architecture. "What happens when
…", "step by step", the order of the calls of one request: a sequence.
Never draw one kind as another without saying so: drawn as architecture,
a request loses its order, and nothing tells the reader.

Each kind this skill draws has a pack, `references/kinds/<kind>/`:
`find.md` says how to find its facts (step 2), and `spec.md` how they
become its spec (step 4). Then:

- a kind archgram lists and this skill has a pack for: draw it;
- a kind archgram lists and this skill has no pack for: say that this
  archgram draws it and this skill cannot yet, and offer a kind it can
  where one answers part of the question;
- a question no listed kind answers, such as the states something goes
  through (a lifecycle) or which role does each step of a business
  process (a workflow in lanes): say that archgram does not draw it yet,
  and offer a kind it draws where one answers part of the question.

An archgram that refuses `--kinds` is older than the listing, such as a
project's own older one. The kinds it draws are then the values
`diagram` takes in `spec --section top-level`, and with no `diagram`
there, architecture alone. A kind this skill has a pack for and that
archgram does not draw needs a newer archgram: say so, and draw nothing
of that kind.

## 1. Decide who reads it

Read `references/reader.md` and write the reader contract: who reads the
diagram, the three to five questions they must answer from it alone, the
one idea it must make obvious, and what is left out on purpose. Everything
after this is checked against it. Take the reader from the context (the
README's audience, the user's words); ask only when nothing tells you.

## 2. Gather the facts

Read the kind's `find.md` (`references/kinds/<kind>/find.md`), then the
code and the docs the diagram describes, and find the facts as it says.
Whatever the kind, draw a part only when a file backs it: a module, a
route, a script, a store. Name it after the file that decides, found by
following the imports, not by a name that looks right. Draw a line
between two parts (an edge, a message) only where a line of code makes
it, and note that line (`src/api.ts:42`) and a few words of it that say
what it does (`db.insert(order)`). Every label and note says what the
code shows, too: what only the README or the docs say ("the data team
reads it") is a claim no line backs, so it goes into the report as a gap
or a question, never into the drawing. Keep the list of parts with their
files and lines with theirs; the report ends with it, and it catches
boxes and lines that exist only in someone's memory. How much of it to
draw, and what to merge, is the kind's `spec.md` (step 4).

### From a document

When the user names a document to draw from (a docs page, a design doc,
an article: `draw the design in docs/design.md`), that document
is the facts, in place of the code; without one named, draw from the
code. Read it whole. Draw a part only where a sentence states it, and an
edge only where a sentence says one part calls, sends to or reads from
another. Note, for each, the sentence's file and a few words from it on
one line of the file (a paragraph wrapped across lines splits a
sentence), such as
`docs/design.md#the worker takes each job from the queue`.
Label each part and edge in the document's own words, not a paraphrase:
a reader who goes from the drawing to the text finds the same names.
What the document does not state is not drawn, however likely; say it in
the report as a gap. When the document and a diagram already beside it
disagree, say where and ask which is right. Everything after this step
holds for a document as for code: the critique's "file" is the
sentence.

Ask the user, and only then, when the architecture is unclear: the README
describes a part the code does not have, two readings of the code are
equally likely, or the request names something you cannot find. Say what
you found and what you need to know. Everything else you decide yourself.

## 3. Learn the spec format from archgram

```bash
npx --yes --loglevel=error archgram@0.11.0 spec --brief
```

This prints the short part of the format the very archgram you run reads:
one complete spec that uses each field most diagrams need, and a line for
each other section. Read it before writing, rather than writing from
memory. When you need more, print that section alone, by the name the
brief gives it: the theme file in step 5, or the section a problem from
`archgram check` points to.

```bash
npx --yes --loglevel=error archgram@0.11.0 spec --section <name>
```

The brief holds an architecture's fields. For any other kind, print the
section `spec --kinds` named for it too, before writing, such as:

```bash
npx --yes --loglevel=error archgram@0.11.0 spec --section sequence
```

A field the format does not list is an error. Besides the fields, it says
how a writer fills some of them, such as where the files and lines from
step 2 go: do as it says. It is the format of the very archgram you run,
so what it does not mention, that archgram does not read: leave it out.

## 4. Write the spec, then check it

The spec lives at `docs/diagrams/<name>.archgram.yaml`, where `<name>` says
what the diagram shows (`architecture`, `request-flow`). Create the folder
when it is missing. When the file already exists, this is an update:
change it rather than starting over, keep what still holds, and note what
you changed for the report; keep, too, what the kind's `spec.md` says an
update keeps (an architecture's direction). Start an update with
`archgram check` on the spec as it is: whatever it says has lost its
code, find where that code went, if anywhere, and move or remove the
part or line. The check cannot see what the code gained, so step 2's
walk still finds what is new. Change what the check and the walk found,
and nothing more: a part or a flow the request did not ask for is the
user's to add.

Write the facts in the format as the kind's `spec.md`
(`references/kinds/<kind>/spec.md`) says, and draw as much of them as it
says a reader takes in.

Say where the drawing is shown when it is not a README on GitHub: a docs
site, a blog, a wiki, any page that shows it narrower. Write that width
as `shownWidth`, in CSS pixels, the place at its widest, from the user's
words or the page's own styles; ask only when nothing tells you. A
drawing shown in two places takes the narrower.

Put step 2's files and lines, or the document's sentences, in the spec
wherever the format has a place for them, as it says, so archgram can
tell when the code or the text behind a part or a line is gone. A line
from someone outside the code, a browser or a person at a terminal,
takes the line of code that receives it.

Then check it, and repeat until it passes:

```bash
npx --yes --loglevel=error archgram@0.11.0 check docs/diagrams/<name>.archgram.yaml
```

Check again after every change to the spec, the last one included, before
step 6 draws it: `build` draws a spec whose code is gone with only a
warning, so only the check holds the final spec to the code.

Every problem comes with its line and column, and a misspelt id or logo
with the nearest one that exists. A problem that says the code lacks what
you wrote is a part or a line you could not back: fix the fact, not only
the spec, and drop it if no code makes it.

## 5. Draw in the project's colours, when asked

archgram draws in black and white by default, and so does the diagram
unless the user asks for colours: the project's own ("in our colours",
"match our design system") or colours they name. The colours are theirs
to choose, as the parts are: a theme guessed from the styling code
changes how their README looks without their asking. When they ask, read
`references/theme.md`, which says where archgram's colour goes, where to
look (design tokens, then the styling code, then colours the user gave)
and how to hand them to archgram with `--theme-file`. Otherwise leave the
styling files unread and write no theme.

An `archgram.theme.json` already at the project's root was asked for
before: keep drawing with it.

## 6. Draw, critique, open

```bash
npx --yes --loglevel=error archgram@0.11.0 build docs/diagrams/<name>.archgram.yaml
```

Add `--theme-file archgram.theme.json` when the project's root has one.
The drawing lands beside the spec as `docs/diagrams/<name>.svg`, and
`build` says its size and the direction it chose: `wrote … (588 × 740 px,
top to bottom)`.
A drawing wider than keeps its text readable where it is shown, 1,300 px
in a README on GitHub (which shows an image about 880 px wide) or
`shownWidth × 1300 / 880` elsewhere, is still drawn, and `build` warns
with what would bring it within: the other direction, with the width it
would draw, or two diagrams. Do what it says: write that direction in the
spec, or split the diagram in two, check each, and build again.

Run the critique in `references/reader.md` against the contract and fix
what it finds: a question the picture cannot answer, a box no file backs,
an edge with no line of code behind it, a missing hop. Judge from the spec
and what `archgram build` wrote; never start a browser, headless or not,
and never take a screenshot: the user sees the drawing, and a browser
started in the background alarms them. Once the drawing passes, do what
the kind's `spec.md` asks of a drawing that passed: an architecture
writes the direction `build` chose.

Then open the drawing for the user, once, with the system's own viewer:
`open` on macOS, `xdg-open` on Linux, `start ""` on Windows.

## 7. Report

Tell the user, briefly, all of the following, on an update too: what
changed comes first there, and the rest still follows.

- the reader contract, and how the drawing answers it;
- where it is drawn to be shown (a README, or `shownWidth`), and its
  size;
- each part drawn, one line each, with the file (or the sentence) behind
  it, or that no file backs it (a browser, a person), and each edge, one
  line each, with the line (or the sentence) that makes it, those an
  update left unchanged included;
- what the kind's `spec.md` adds to the report, or puts in place of the
  parts and edges (a sequence's participants and messages);
- on an update, what changed in the spec and why, starting with what
  `archgram check` said had lost its code;
- where the colours came from, or that they are archgram's own and the
  project's can be asked for;
- what the diagram still does not show;
- the line for the README, with alt text that tells the whole flow in
  words: `![<the flow in one sentence>](docs/diagrams/<name>.svg)`.

Edit the README, or commit, only when the user asks; the project's own
rules (AGENTS.md, CLAUDE.md, CONTRIBUTING) decide how.
