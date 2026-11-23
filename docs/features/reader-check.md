# Feature: check that a reader understands the drawing

| Field   | Value      |
|---------|------------|
| Version | 0.2        |
| Date    | 2026-11-20 |
| Status  | Draft      |
| Release | Planned    |
| Issue   | [#173](https://github.com/byAbas/archgram/issues/173) |

What this feature does and why. `docs/PRD.md` (§6.5, §6.6) points here;
how the preview is drawn goes into `ARCHITECTURE.md`, what it reads into
`SECURITY.md`, and the option into `README.md`.

## Problem

Every diagram is meant to be understood by someone who did not build the
system, within thirty seconds (`skills/archgram/references/reader.md`).
Nothing checked that it is. The engine's tests hold the geometry (no line
through a card, no label over another), the specs' sources are held to
the code, and the facts to the truth where a test knows it; what a reader
sees was checked by no one but a person looking.

The skill had a critique for it, "is the one idea visible", "does a line
detour", but could not run it: its step 6 forbade starting a browser or
taking a screenshot, after #14, where it had started a headless browser
on its own and the user saw an unexpected prompt, and nothing took the
place of looking. `build` says only a drawing's size and direction. So the
critique of what a reader sees was answered from the spec.

### Evidence

- On `nextjs/saas-starter` (2026-11-19), the skill chose as its one idea
  "Stripe owns the subscription" and reported that two lines into
  Postgres "show the main point", adding "I haven't looked at the image
  myself; I judged the layout from the spec and the build output". In the
  drawing, Stripe stands at the far right, beside the server's frame, and
  the eye lands on Postgres, where five arrowheads meet. The facts were
  right: 14 of 14 lines true, 27 of 27 sources found.
- The evaluations held the same gap: eight cases expected "No browser was
  started and no screenshot taken", and none what a reader of the image
  understands.
- The first previews drawn with this feature showed what no one had
  seen: in the dark theme, the PostgreSQL logo's blue (`#4169e1`) is hard
  to see on the dark card.

## Who it serves

| User | What they get |
|---|---|
| The reader of the diagram | A drawing whose questions were answered from the image itself before it reached them |
| The user of the skill | A report of what the skill saw, and a drawing it changed after looking, not one it assumed |
| Whoever wants a still image | A PNG of each theme, for a page or a print that shows no SVG |

## Requirements

### The engine

- `archgram build <spec> --preview` also writes a still PNG of each theme,
  `<name>.light.png` and `<name>.dark.png`, under `.archgram/preview/`
  at the project's top, and says where
  ([approved-drawing.md](approved-drawing.md)). It works with every other
  option of `build`, and for both kinds of diagram.
- Each preview is rendered inside archgram, with resvg
  (https://github.com/linebender/resvg), from the SVG archgram draws in
  that theme, at the drawing's size rounded up, so nothing of it is cut
  off. Two things are done before resvg reads it, since its parser leaves
  them out: each CSS custom property (`var(--card)`) is replaced by the
  value the drawing's style gives it, and the embedded font's family,
  `archgram-geist` and `archgram-geist-mono`, is named as the font files
  name it. A property given no value, or two, stops the preview with an
  error, never a guess: an SVG whose colours resvg cannot read draws
  black.
- It is the still image: no animation runs, and every element that starts
  hidden is an animation's overlay (signals, lit borders, refusals), so
  the preview shows the diagram as `prefers-reduced-motion` does
  (DESIGN.md), the image that must carry the whole meaning.
- It reads nothing but that SVG: only archgram's embedded fonts are
  loaded, never the system's; no image, file or data URL is fetched, and
  archgram's SVG refers to none; there is no network and no browser.
- resvg is the one dependency it adds, with `text` its only feature; its
  licence (Apache-2.0 OR MIT), owners and advisories are in #173, and
  `cargo deny` holds it and what it brings as it holds every dependency.

### The skill

- Step 6 builds with `--preview`, looks at both previews and runs the
  critique of `references/reader.md` on what it sees, not on the spec:
  each of the reader contract's questions answered from the image alone,
  the one idea's parts where the eye lands and not pushed to an edge,
  arrows that crowd or cross, and anything hard to read in either theme.
  It fixes what it finds in the spec, builds and looks again, and the
  report says what it saw, each question's answer, and what it changed.
- It still starts no browser, headless or not.
- With an archgram older than the preview, which refuses `--preview`, it
  builds without it and says in the report that it could not see the
  drawing.
- The previews are for checking, not for the project; git leaves their
  folder out, so the report says nothing of them.

### The evaluations

- A case that draws expects that no browser was started, and that the run
  built with `--preview`, looked at both previews and reported what it
  saw.
- **The reader test.** A case with `reader` questions gets, after a run
  with the skill, a fresh session in an empty folder that holds only the
  run's light preview; it answers each question from the image alone, and
  an answer is right when it holds one of the words a right one holds.
  The score is kept beside the run (`evals/archgram/README.md`).

## Limits

- The preview is the still image: an animation's order and timing are not
  in it.
- Its pixels are resvg's, not a browser's: the layout is the drawing's
  own, the antialiasing and text a little different.
- A drawing of more than 32 million pixels gets none: the build stops
  with an error rather than take gigabytes. A drawing of the most nodes
  a spec may have, in a row, is about 12.5 million; only very long labels
  make one larger, and it is too large to read.
- A reader test checks only the questions a case asks, and scores an
  answer by its words, not its sense.
- The skill sees the previews only where its agent reads images.

## Not in this

- A report of the layout in text (where each card stands, which sit
  beside a frame, how many lines cross), for agents that do not read
  images.
- Comparing the preview with a browser's rendering of the same SVG, in CI.
- Changing the layout itself, such as where a part outside a frame
  stands: what the critique finds, the skill answers in the spec (its
  direction, its hints, two diagrams).

## Success criteria

- `build --preview` draws every example in both themes at the drawing's
  size, its canvas the theme's own colour and almost none of it black
  (`build_preview_draws_each_theme_as_the_drawing_shows_it`).
- A property with no value or two stops the preview with an error.
- In the skill's evaluation, every case that draws reports what it saw in
  the previews, and the reader test scores the questions it asks.
- On `nextjs/saas-starter`, the critique names that Stripe, the one idea,
  stands at an edge, and the report says what the skill did about it.

## Decided (2026-11-19)

1. **The skill looks at its drawing.** #14's rule against a browser
   stays; the rule against looking goes. The owner's decision: the result
   is checked end to end without loosening security or privacy.
2. **resvg renders it, inside archgram.** The standard library draws no
   image; resvg is the most used SVG renderer in Rust (about 40 million
   downloads), with no advisory, `#![forbid(unsafe_code)]` in `resvg` and
   `usvg`, and no C library, where librsvg parses XML with libxml2 and
   has a use-after-free advisory this year (RUSTSEC-2026-0305). Drawing
   archgram's own scene with `tiny-skia`, or a rasterizer of archgram's
   own, would put the icons and the text in our hands, and any error there
   into the preview.
3. **One preview per theme, inside the project**, so the critique sees the
   dark theme too, and the skill's agent reads them where it asks no
   permission to read; since 2026-11-20 in `.archgram/preview/`, out of
   git ([approved-drawing.md](approved-drawing.md)).

## Open questions

- Whether more of the layout in text should follow, for agents that do
  not read images, beyond the counts `build` prints
  ([approved-drawing.md](approved-drawing.md)).
- Whether CI should compare each example's preview with a browser's
  rendering, and with what tolerance.

## Changelog

| Version | Date       | Change |
|---------|------------|--------|
| 0.1     | 2026-11-19 | First draft, from #173: `build --preview` writes a still PNG of each theme, rendered with resvg after the custom properties and the font's family are resolved; the skill's step 6 looks at it and critiques what a reader sees; the evaluations expect it, and a reader test answers each case's questions from the image alone. |
| 0.2     | 2026-11-20 | The previews go under `.archgram/preview/` at the project's top, out of git, not beside the drawing, and the report says nothing of them; that answers whether the skill should leave them out of git (#176). |
| 0.3     | 2026-11-23 | A drawing of more than 32 million pixels gets no preview, so a spec cannot make a build take gigabytes (Limits). |
