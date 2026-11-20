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

Run each command on its own, as it is written here: not joined to another
with `&&`, `;` or a pipe, and not inside a loop. An agent may approve the
exact commands this skill names, and a joined one asks the user or is
refused. Read files by reading them, one at a time, and change the spec
by editing it, not through the shell.

```
Progress:
- [ ] archgram found, or the user asked
- [ ] The kind of diagram
- [ ] 1. Reader contract
- [ ] 2. Facts written in the facts document, and approved
- [ ] 3. The spec format, from archgram itself
- [ ] 4. Spec written and checked
- [ ] 5. The project's colours, when asked
- [ ] 6. Drawn, looked at, critiqued, opened
- [ ] 7. Reported
```

## Find archgram, and ask before downloading it

Every command below runs archgram through npx, which needs Node 22 or
later. npx downloads a package it does not find, and with no terminal
attached it does so without asking. So this skill runs archgram from
where it already is, and downloads it only when the user says so. Do
this once, before the first command. First read the project's root:
whether it has a `package.json`, and whether that lists archgram in its
dependencies or dev dependencies. What follows, the question included,
goes by what you read there, never by a guess.

- **The project's own.** When the project's `package.json` lists
  archgram, run that one: write each command below with `archgram`
  where it says `archgram@0.11.0`, and `--no --offline` where it says
  `--yes`. npx then runs what the project installed, at the version its
  lockfile decides, and downloads nothing. If npx says a package is
  missing, the project's dependencies are not installed: ask the user to
  install them with the project's own package manager, and wait. Run the
  version check below the same way: when the project's archgram is older
  than `archgram@0.11.0`, the version this skill was released with, draw
  with it all the same, and say so in the report (step 7).
- **This version, already on the machine.** Otherwise, look for the
  version this skill was released with, which an earlier download the
  user agreed to leaves in npx's cache:

  ```bash
  npx --no --offline --loglevel=error archgram@0.11.0 --version
  ```

  It uses no network and installs nothing. When it prints
  `archgram 0.11.0`, run the commands below as they are written.
- **Not there: ask.** Otherwise stop, and ask the user before anything is
  downloaded. Say what and from where: the npm package `archgram@0.11.0`
  and the native archgram program for this system, a few megabytes, from
  the npm registry. Offer two ways, and run neither until the user picks
  one; with no answer, draw nothing:
  - install it into the project, when the `package.json` you read is
    there, with the project's own package manager, such as
    `npm install --save-dev archgram@0.11.0`: its lockfile then pins it,
    and the commands run as for the project's own;
  - let npx download it once, now: the commands below then run as they
    are written.

Keep `--loglevel=error` in every command: it keeps npm's own warnings,
such as those about a pnpm project's `.npmrc`, out of archgram's output,
and changes nothing else. If `npx` is missing, or cannot find that
version on npm, say so and stop.

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
The contract goes into the facts document, at the top of this diagram's
layer (step 2).

## 2. Find the facts, and have them approved

Read `references/facts.md`, `references/ignore.md` and the kind's
`find.md` (`references/kinds/<kind>/find.md`).

Before reading the code, measure what you would read, with the skill's
own `scripts/size.mjs` (the skill's folder is the one this SKILL.md is
in), run with Node from the project's root:

```bash
node <the skill's folder>/scripts/size.mjs
```

It reads the files git tracks, leaves out what `references/ignore.md`
says, and prints, with no network and no file's contents, the size of
what is left (small, medium or large against one reader's budget), its
areas, what it left out and why, and the manifests it saw. When the
scope is narrower than the project, one entry point or one service, run
it again with those folders after it. Tell the user, in a line, the size
and the areas you will read. For a large scope, ask whether to narrow it
before reading it all. Read area by area, and write each area's facts
into the document before the next, so they are kept there rather than
in your memory of them. What it left out goes into the document's
coverage.

Then read the code and the docs the diagram describes, leaving out what
`references/ignore.md` says, and find the facts as the kind's `find.md`
says. When the facts document
(`docs/diagrams/facts.md`) or a spec beside it is already there, this is
an update: start from them, as `references/facts.md` says (On an
update). An update, a repair included, writes the facts document too:
when a spec is there and the document is not, create the document from
the spec's own parts and sources first, then change it as the check and
the walk find, and only then change the spec from it.

Whatever the kind, a part is a fact only when a file backs it: a module,
a route, a script, a store. Name it after the file that decides, found by
following the imports, not by a name that looks right. A line between
two parts (an edge, a message) is a fact only where a line of code makes
it: open that line, and note it (`src/api.ts:42`) and a few words of it
that say what it does (`db.insert(order)`). Every label and note says what
the code shows, too: what only the README or the docs say ("the data team
reads it") is a claim no line backs, level 4 evidence, so it is an open
question, never something to draw.

Write the facts in the facts document as `references/facts.md` says: the
project's parts, boundaries, entry points and what was read in its core,
and in this diagram's layer its reader, its kind's tables, what was
expected and not found, and the open questions. Each fact carries its
evidence, and each line its source. The document holds what the code
says; how much of it to draw, and what to merge, is the kind's `spec.md`
(step 4).

### Approve the facts

Then stop, and ask the user to approve the document before any spec is
written. Give its path, and in a few lines the parts and the lines this
diagram would draw, what was not read, what was expected and not found,
and each open question. When they correct a fact, check it in the code,
change the document, and ask again; on an update, ask about what
changed. Write no spec until they approve. When the request itself says
to go on without stopping, such as "no need to check the facts with me",
go on, and say in the report that the facts were not approved.

An open question stops the drawing either way. Ask the user, and only
then, when the architecture is unclear: the README describes a part the
code does not have, two readings of the code are equally likely, or the
request names something you cannot find. Say what you found and what you
need to know. Everything else you decide yourself.

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
disagree, say where and ask which is right. The facts still go into the
facts document, each sentence as its source and its evidence, and are
approved as above. Everything after this step holds for a document as
for code: the critique's "file" is the sentence.

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
change it as the facts document changed (step 2) rather than starting
over, keep what still holds, and note what you changed for the report;
keep, too, what the kind's `spec.md` says an update keeps (an
architecture's direction). Change what the document's update found, and
nothing more: a part or a flow the request did not ask for is the user's
to add.

Write the spec from the approved facts document, this diagram's layer
and the core's parts it names: only a fact at evidence 1 or 2, each with
the source the document gives it, in the format as the kind's `spec.md`
(`references/kinds/<kind>/spec.md`) says, and as much of them as it says
a reader takes in. A fact the document does not hold is not drawn: find
it, add it to the document and have it approved first.

Say where the drawing is shown when it is not a README on GitHub: a docs
site, a blog, a wiki, any page that shows it narrower. Write that width
as `shownWidth`, in CSS pixels, the place at its widest, from the user's
words or the page's own styles; ask only when nothing tells you. A
drawing shown in two places takes the narrower.

Put the facts document's sources, its files and lines or a named
document's sentences, in the spec wherever the format has a place for
them, as it says, so archgram can
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

## 6. Draw, look at it, critique, open

```bash
npx --yes --loglevel=error archgram@0.11.0 build docs/diagrams/<name>.archgram.yaml --preview
```

Add `--theme-file archgram.theme.json` when the project's root has one.
The drawing lands beside the spec as `docs/diagrams/<name>.svg`, and
`build` says its size and the direction it chose: `wrote … (588 × 740 px,
top to bottom)`. `--preview` also writes a still image of each theme
beside it, `<name>.preview.light.png` and `<name>.preview.dark.png`,
which archgram renders itself: no browser, no network.
A drawing wider than keeps its text readable where it is shown, 1,300 px
in a README on GitHub (which shows an image about 880 px wide) or
`shownWidth × 1300 / 880` elsewhere, is still drawn, and `build` warns
with what would bring it within: the other direction, with the width it
would draw, or two diagrams. Do what it says: write that direction in the
spec, or split the diagram in two, check each, and build again.

Look at both previews, as the reader will see the drawing, and run the
critique in `references/reader.md` on what you see, not on the spec:
answer each of the contract's questions from the image alone, and find
where the one idea's parts stand, where arrows crowd or cross, and what
is hard to read in either theme. Fix what it finds in the spec, build
again with `--preview`, and look again, until it passes; say in the
report what you saw and what you changed. Never start a browser,
headless or not: the previews are what you look at. Once the drawing
passes, do what the kind's `spec.md` asks of a drawing that passed: an
architecture writes the direction `build` chose.

An archgram older than the preview refuses `--preview`: build without it,
critique from the spec and the size as well as you can, and say in the
report that you could not see the drawing.

The previews are for checking, not for the project: tell the user they
can delete them, or leave them out of git.

Then open the drawing for the user, once, with the system's own viewer:
`open` on macOS, `xdg-open` on Linux, `start ""` on Windows.

## 7. Report

Tell the user, briefly, all of the following, on an update too: what
changed comes first there, and the rest still follows.

- the reader contract, and how the drawing answers it;
- the facts document's path, and that the user approved it, or that the
  request said to go on without it;
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
- what you saw in the previews, the answer to each contract question
  the image gave, and what you changed after looking;
- what the diagram still does not show;
- when the project's own archgram is older than `archgram@0.11.0`: its
  version, that this skill was released with `archgram@0.11.0`, and the
  project's own command to upgrade it, such as
  `npm install --save-dev archgram@0.11.0`; upgrade it only when the
  user says so;
- the line for the README, with alt text that tells the whole flow in
  words: `![<the flow in one sentence>](docs/diagrams/<name>.svg)`.

Edit the README, or commit, only when the user asks; the project's own
rules (AGENTS.md, CLAUDE.md, CONTRIBUTING) decide how.
