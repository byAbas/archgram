# Feature: draw what was approved, so a reader can follow it

| Field   | Value      |
|---------|------------|
| Version | 0.1        |
| Date    | 2026-11-20 |
| Status  | Draft      |
| Release | Planned    |
| Issue   | [#176](https://github.com/byAbas/archgram/issues/176) |

What this feature does and why. `docs/PRD.md` (§6.3, §6.5, §6.6, §7)
points here; the title and the subtitle go into `docs/SPEC.md`, how they
look into `DESIGN.md`, the layout report and the preview's folder into
`ARCHITECTURE.md` and `README.md`, the logos' licences into
`THIRD-PARTY-LICENSES`, and how the skill runs it into `skills/archgram/`.

## Problem

The skill finds a project's facts, writes them down, has the user approve
them, and draws them (`docs/features/facts.md`). Three runs on projects
whose architecture the code shows showed that the facts are right but
what is drawn can drift from them after approval, and can be hard to
follow:

- After approval, a split into two diagrams changed what the user had
  approved: new sources, new labels, a new main point. Nothing checks
  that what is drawn is what was approved.
- A relation the code backs can be left out with no reason given, and
  the flow then breaks where the reader would ask how it got there.
- A drawing can mix what runs with how it is built, run lines back up the
  page, cross frames it has nothing to do with, or bring lines from two
  sources side by side into one card. archgram measures none of it, so
  the skill cannot see it except by eye.
- A drawing does not say what it shows.
- The previews are left in the project; the report is long, names a
  README nobody asked for, and lists every part and line.
- Well-known technologies have no logo.

### Evidence

All three on 2026-11-20, archgram 0.11.0, the prompt "Draw the
architecture of this project", the user approving the facts as asked:

- **GoogleCloudPlatform/microservices-demo.** Every relation true, 16 of
  16 services found. The approved one diagram was split into two without
  saying so. Rebuilt as one diagram (1,651 × 736 px), the frontend's lines
  cross checkoutservice's trunk just before their arrowheads; another
  agent given only the image named 16 of 17 edges, missing
  frontend → shippingservice, where two sources arrive side by side.
- **fastapi/full-stack-fastapi-template.** Every relation true. 4 of 9
  boxes are build steps drawn with the same arrows as requests; two
  cycles; four lines cross a frame's border; one crossing a layout hint
  could not remove; a file copy ("static build") drawn as a call.
- **nextjs/saas-starter.** Every relation true. Split after approval, and
  said so; the split gave approved relations two new sources and two new
  labels ("SQL" became "update teams"). The billing diagram leaves out the
  browser's redirect to Stripe's hosted Checkout
  (`lib/payments/stripe.ts`, `redirect(session.url!)`), which the facts
  list as not drawn but the reader contract does not leave out; its title,
  "Billing write-back", drops the project's name.
- **All three.** Each `check` first failed on a comma inside `{ }` (3 of
  3), an `order` hint across frames (2 of 3), or a logo archgram does not
  carry (2 of 3: `openjdk`, `gnubash`), each fixed in one retry. Each
  report ran past 40 lines with tables of every part and line, said
  "drawn for a README on GitHub", and asked the user to delete the
  previews from `docs/diagrams/`.
- **Logos.** archgram 0.11.0 carries Simple Icons' CC0 logos only; of release
  16.33.0's 223 logos under a licence of their own, 11 are CC0 and carried,
  212 left out: among them JavaScript, Rust, PHP, Ruby, Git, OpenJDK, GNU
  Bash, Apache Kafka, Angular and Vue.js.
- The one drawing the owner named clean, microservices-demo's "placing an
  order": one question, 8 boxes and 8 lines, no crossing, no line running
  back, one line into the frame, each card reached by one line.

## Who it serves

| User | What they get |
|---|---|
| The reader of the diagram | A titled drawing of one idea, its lines followed without guessing |
| The user of the skill | What they approved is what is drawn, or the skill stops and asks; a report they read in a minute |
| Anyone drawing a well-known technology | Its logo |

## Requirements

### The engine

- **A visible title.** Every drawing shows, above it, "<Kind> diagram of
  <title>", the kind from the spec's `diagram` ("Architecture diagram of
  Online Boutique", "Sequence diagram of sign in"), in English. An
  optional `subtitle`, one line of text, is shown under it. The spec
  hides both (`heading: false`). The SVG's `<title>` and `<desc>` are
  unchanged, and a screen reader still hears the drawing as one image.
- **The layout report.** `build` prints, after the size and the
  direction, four counts from the lines as they are drawn: lines that
  cross, lines that pass through a frame they neither start nor end in,
  lines that run against the main direction, and targets reached by lines
  from different sources running side by side into them. Text only; the
  SVG is unchanged.
- **The previews' folder.** `--preview` writes its PNGs under
  `.archgram/preview/` in the project's top folder, the nearest one above
  the drawing that holds `.git`, else the drawing's own, at the drawing's
  path inside it (`.archgram/preview/docs/diagrams/architecture.light.png`).
  archgram writes a `.gitignore` of `*` in `.archgram/` when there is
  none, so git ignores the folder without a change to the project's own
  `.gitignore`.
- `archgram --help` lists `--preview`.
- **Every logo.** archgram carries every logo of its pinned Simple Icons
  release, whatever its licence. Each logo's licence is recorded with its
  source and guidelines; `THIRD-PARTY-LICENSES`, the `archgram-icons`
  crate's `license` and `cargo deny`'s allowed licences name every licence
  it ships.
- **Logo contrast, measured.** The change reports how many logos, and
  which drawings among the goldens, a 4.5:1 limit against the card would
  change; the 3:1 limit stays.

### The skill

- **The request first.** The user's words decide what is drawn, how many
  diagrams, which phase and which level. Where they say nothing, the
  limits below apply; where a limit conflicts with what was asked, the
  skill draws what was asked and reports the limit's numbers with one
  line on the cleaner way.
- **Held to approval, by a script.** On approval, a script of the skill's
  stamps each approved part of `docs/diagrams/facts.md`. Before each
  build, it checks, and the skill stops and asks when one fails:
  - the approved text is unchanged; a split adds a diagram of its own,
    "split from" the approved one, naming the questions it answers;
  - every approved part and relation is in at least one spec, and every
    question in at least one diagram;
  - every edge in a spec is a relation in the facts, its label the
    approved label;
  - every relation the facts list as not drawn is in the reader
    contract's "left out on purpose", with its reason.
  A new source for an approved relation needs no approval when `check`
  finds it.
- **One idea per drawing.** The facts give each part a phase (run, build,
  setup), and one diagram draws one phase; another phase is offered as a
  diagram of its own. A reader contract asks at most three questions,
  counted by the script. The layout report's counts are 0, or the skill
  changes the spec until they are, and reports them when it cannot.
- **Where it is shown, from the user.** The facts say where the drawing
  is shown from the user's words, or "not said" with the default width;
  the reader comes from the user's words, else the project. "README" is
  written only when the user names one.
- **What archgram could not draw.** Each diagram's part of the facts lists
  what the skill wanted and archgram could not do, the version that could
  not, and what was drawn instead, never a rule `docs/SPEC.md` states or a
  mistake of the skill's own; an update tries each again with a newer
  archgram.
- **A short report**, about 15 lines: each drawing's path and what it
  shows; how exact it is (`check`'s count, the facts' path, whether they
  were approved); at most three things changed after looking; at most
  three not shown; at most one question. No list of parts or lines.
- **Fewer failed checks.** The spec's rules for the skill say: quote
  every `note` and `label` inside `{ }`; an `order` hint orders the nodes
  of one frame; `tech` only with a logo archgram carries.
- **The critique follows each flow** from its first step to its last, and
  asks where a reader would ask how it got there.

### The evaluations

- A case that draws expects a report under 20 lines that names each
  drawing's path and the facts' path and holds no table of parts or
  lines; no "README" unless the prompt names one; the facts script run
  and passing; and the layout report's counts in the report when they
  are not 0.
- The next lab round, run by hand on the three projects and on the
  owner's budget, asks for something specific: the whole system in one
  diagram with its build (drawn as one, its counts reported); how CI
  deploys (fastapi-template); the backend's modules only; the billing
  only (saas-starter).

## Limits

- The counts say how many, not which line; the critique still looks.
- "Side by side" is two arrivals no more than `2 × spacing.edge-edge`
  (24 px) apart (ARCHITECTURE.md, Layout); two just farther apart are not
  counted. Every card reached from more than one source is counted, the
  README's linkshort among them: one line into each card is the target.
- The script checks the facts' text and the specs, not that a relation is
  true: `check` and the evidence levels do that.
- A logo's licence is recorded, not checked against how a drawing is
  used: that stays with whoever publishes it, as its brand's guidelines
  do (§7).

## Not in this

- `direction: auto` weighing the counts: decided from their numbers once
  measured.
- Changing the 3:1 logo limit.
- A title in another language.

## Success criteria

- The microservices-demo spec rebuilt as one diagram reports at least one
  target reached side by side; "placing an order" reports 0 crossings, 0
  lines through a frame, 0 lines against the direction, 0 side by side.
- On the three projects, with the prompt above: nothing approved changes
  without the skill stopping; every relation the code backs and the
  drawing leaves out has its reason; each report is under 20 lines;
  `check` fails at most once per run.
- Every golden shows its title, and every example builds.
- `javascript`, `rust`, `openjdk` and `gnubash` are logos archgram draws.

## Decided (2026-11-20)

1. **The order:** accuracy never bends; then the user's request; then the
   limits, only where the request says nothing.
2. **The title is composed** by the engine, shown by default; the
   subtitle only where two diagrams would share a title.
3. **A split is the skill's,** without asking again, held by the script.
4. **Previews outside git's view,** in `.archgram/preview/`.
5. **Every logo,** whatever its licence: the owner's decision, with the
   risks of the copyleft and non-commercial licences shown.
6. **The heading has no symbol,** and takes a node's name's type, so the
   names keep the one weight step (DESIGN.md, Typography).
7. **Side by side is any parallel arrival,** no run required: measured on
   the single microservices-demo diagram, a run of 52 px or more counted
   nothing where the reader missed a line; any arrival counted it.

## Open questions

None.

## Changelog

| Version | Date       | Change |
|---------|------------|--------|
| 0.1     | 2026-11-20 | First draft, from #176: a visible title; `build`'s layout report; previews in `.archgram/preview/`; every Simple Icons logo; the skill held to the approved facts by a script, one idea and one phase per drawing, a short report, the place shown taken from the user. |
