<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/byAbas/archgram/main/docs/images/logo-dark.svg">
    <img src="https://raw.githubusercontent.com/byAbas/archgram/main/docs/images/logo.svg" alt="" height="96">
  </picture>
  <h1 align="center">archgram</h1>
</p>

<p align="center">
  <a aria-label="npm version" href="https://www.npmjs.com/package/archgram"><img alt="npm version" src="https://img.shields.io/npm/v/archgram.svg?style=for-the-badge&labelColor=000000"></a>
  <a aria-label="License" href="https://github.com/byAbas/archgram/blob/main/LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge&labelColor=000000"></a>
  <a aria-label="Website" href="https://archgram.dev"><img alt="Website: archgram.dev" src="https://img.shields.io/badge/website-archgram.dev-000000.svg?style=for-the-badge&labelColor=000000"></a>
</p>

Aesthetic architecture diagrams, straight from the code. Animated flows,
light and dark.

archgram is an open-source CLI tool for software architecture diagrams.
Write a YAML or JSON spec, or let Claude Code, Codex, Cursor or Gemini CLI
draw one from your code. It writes one self-contained SVG that follows
light and dark mode and animates the path a request takes. The file drops
into a GitHub README or any docs page, and the animation plays wherever an
image shows.

![archgram's own architecture: a developer runs the archgram CLI, or asks a coding agent, which loads the diagram skills; the skills build the diagram with the CLI, and fetch archgram from the npm registry only when the project does not have it. The animation follows one prompt to a diagram.](https://raw.githubusercontent.com/byAbas/archgram/main/docs/diagrams/architecture.svg)

archgram's own architecture, drawn by its skill from `Draw the architecture
of this project.` and nothing more
([the spec](https://github.com/byAbas/archgram/blob/main/docs/diagrams/architecture.archgram.yaml)).

You describe the system: its nodes, what connects them and the paths a
request takes. archgram lays it out, routes the lines around the boxes and
draws it, in light and dark, with the flows animated.

It also draws a sequence diagram: the order in which the parts call each
other for one request, and what comes back, told in phases and played one
phase at a time ([docs/SPEC.md,
Sequence](https://github.com/byAbas/archgram/blob/main/docs/SPEC.md)).

## Install

```sh
npm install --save-dev archgram
```

The command is a native binary for macOS, Linux and Windows, on x64 and
arm64. npm installs the one for your machine from an optional dependency,
`@archgram/cli-<os>-<cpu>`; nothing runs or downloads at install time.

If npm was told to leave optional dependencies out (`--omit=optional`),
or the lockfile was written on another platform, the binary is missing:
delete `node_modules` and `package-lock.json` and run `npm install` again.
`ARCHGRAM_BINARY` points the command at another binary.

## Use

Write a spec in JSON or YAML, for example
`docs/diagrams/linkshort.archgram.yaml`:

```yaml
archgram: 1
title: linkshort
description: A visit goes through the API to Postgres.
nodes:
  - { id: browser, kind: browser, label: Visitor }
  - { id: api, kind: service, label: API, tech: fastapi }
  - { id: db, kind: database, label: links, tech: postgresql }
edges:
  - { from: browser, to: api }
  - { from: api, to: db }
flows:
  - { name: a visit, steps: [browser, api, db] }
```

Then draw it:

```sh
npx archgram build docs/diagrams/linkshort.archgram.yaml
```

It writes `docs/diagrams/linkshort.svg` beside the spec, light and dark in
one file.

## Commands

### `archgram build <spec>`

Draws the diagram. A spec named `<name>.archgram.yaml` (or `.yml`,
`.json`) draws `<name>.svg` beside it. It says the size it drew and the
direction, then, for an architecture, what makes its lines hard to
follow: `layout: 0 crossings, 0 lines through a frame, 0 lines running
back, 0 cards reached side by side`. It warns, still drawing, when the drawing is wider than keeps
its text readable where it is shown: 1,300 px in a README on GitHub, or
the ratio of `shownWidth` when the spec says it is shown elsewhere. With
`direction: auto` in the spec, archgram keeps left to right while it
fits, and otherwise the narrower direction ([docs/SPEC.md](docs/SPEC.md)).

```sh
npx archgram build docs/diagrams/linkshort.archgram.yaml
```

| Option | What it does |
|---|---|
| `-o <file.svg>` | Writes another file, and creates its folder when it does not exist |
| `--theme auto\|light\|dark` | Which theme the file carries: `auto`, the default, both, following the reader's dark mode; `light` or `dark` one only |
| `--split-themes` | Writes `<name>.light.svg` and `<name>.dark.svg`, for a page that picks one per reader |
| `--theme-file <archgram.theme.json>` | Draws in your design system's colours, from its design tokens (W3C Design Tokens) |
| `--system-font` | Leaves the text to the reader's font instead of embedding Geist |
| `--preview` | Also writes a still image of each theme, `<name>.light.png` and `<name>.dark.png`, to see the drawing as a reader does, under `.archgram/preview/` at the project's top (the nearest folder above the drawing that holds `.git`), at the drawing's path inside it; `.archgram/` gets a `.gitignore` of `*`, so git leaves it out. archgram renders it itself, with no browser |

### `archgram check <spec>`

Checks a spec without drawing it, and lists every problem at its line and
column, with the nearest id or logo when one is misspelt. When a node or
an edge names what backs it (`source`), its code or the document that
states it, it also says which of that is gone: a part or a line whose
code or sentence was removed is reported, and `archgram build` warns of
the same and still draws.

```sh
npx archgram check docs/diagrams/linkshort.archgram.yaml
```

### `archgram spec`

Prints the spec format this archgram reads: every field, node kind and
rule. `--brief` prints its short part, one complete spec and what each
other section covers; `--section <name>` prints one section, such as
`theme-file`. `--kinds` lists the kinds of diagram it draws, one a line:
the value a spec's `diagram` takes, the section that holds its fields and
the question it answers.

```sh
npx archgram spec --brief
```

### `archgram logos <name>...`

Prints the logo each technology's name stands for, as a node's `tech`
takes it: a product's name, a package's or an image's. When a name
stands for several logos it lists them; when for none, it says so.

```sh
npx archgram logos sentry-sdk Next.js postgres
```

### `archgram guide`

Prints the rules the `draw-<kind>-diagram` skills share; `--section
<name>` prints one section.

### `archgram theme check <archgram.theme.json>`

Reads your design tokens as the theme and shows each colour role in light
and dark, or every problem, such as a pair that falls short of contrast.

```sh
npx archgram theme check archgram.theme.json
```

### `archgram --version`, `archgram --help`

Print the version, or every command and option.

## Draw with your coding agent

Two skills let a coding agent draw from a project's code, one for each
kind of diagram:

| Skill | Ask | Draws |
|---|---|---|
| `draw-architecture-diagram` | `Draw the architecture of this project.` | What the system is made of and what calls what, at the C4 container level |
| `draw-sequence-diagram` | `Draw the sequence of signing in.` | The order in which the parts handle one request, and what comes back |

From one prompt, a skill reads the code, writes the spec in
`docs/diagrams/`, checks and draws it with `npx archgram`, in archgram's
colours or, when you ask, your project's own, and reports in a few lines
what it drew and how exact it is, each part's file and each line's code
kept in the spec's sources. It asks only before it downloads anything: it runs
the archgram your project lists, or the skill's version already on your
machine, and otherwise asks whether to install that version into your
project or to let npx download it once. It is
written to the open [Agent Skills](https://agentskills.io) format, so any
agent that reads skills can use it: Claude Code, Codex, Cursor, GitHub
Copilot, Gemini CLI and others. Install it with the
[`skills`](https://github.com/vercel-labs/skills) command, which puts it
where each of your agents looks:

```sh
npx skills add byabas/archgram
```

It asks which agents to install them for; add `-g` to install them for
every project rather than this one. Then ask your agent as the table
shows, or, in Claude Code, type `/archgram:draw-architecture-diagram`.

In Claude Code, the two skills are also one plugin, `archgram`, in
[Anthropic's directory](https://claude.ai/directory): find archgram there
and add it to your claude.ai account. Claude Code loads it, as
`archgram@synced`, the next time it starts signed in with that account
([plugins synced from claude.ai](https://code.claude.com/docs/en/plugins/loading#synced-plugins)).

Without a claude.ai account, with an API key, Bedrock or Vertex, install
the plugin from this repository, which is also its marketplace, in a
Claude Code session (2.1.275 or later):

```text
/plugin install archgram --marketplace byAbas/archgram
```

It installs as `archgram@archgram`, and Claude Code loads it in place of
`archgram@synced` when both are on
([name conflicts](https://code.claude.com/docs/en/plugins/loading#name-conflicts)).
A marketplace of your own does not update by itself: run
`claude plugin update archgram@archgram` for each release
([updates](https://code.claude.com/docs/en/plugins/install#keep-plugins-updated)).

Or take them from the release, into your agent's skills folder
(`~/.agents/skills`, or `~/.claude/skills` for Claude Code):

```sh
mkdir -p ~/.agents/skills && curl -sL https://github.com/byAbas/archgram/releases/download/v0.13.1/archgram-skill-0.13.1.tar.gz | tar -xz -C ~/.agents/skills
```

The skills' archive carries a signed record of the build that made it:
`gh attestation verify archgram-skill-X.Y.Z.tar.gz -R byAbas/archgram`.
0.5.0 and earlier were built before the account was renamed, and their
records carry its earlier name.

## FAQ

### Does the animation play in a README or a docs page?

Yes. The flows are animated with SMIL inside the SVG, which needs no
script, so the drawing moves in an `<img>`: in a GitHub README, on npm and
in any docs page.

### Does it follow dark mode?

One SVG carries both themes and follows the reader's system setting. To
follow GitHub's own theme instead, draw with `--split-themes` and put the
two files in a `<picture>` with `prefers-color-scheme`.

### Does it need a server, a browser or the network?

No. archgram is one native binary, and the SVG is self-contained: no
script, no external file, no web font, so it shows the same wherever it is
opened.

### How is it different from Mermaid?

Mermaid draws many kinds of diagram from text and is rendered by the page
that shows it; GitHub renders it in Markdown. archgram draws two kinds,
the architecture of a system and the sequence of one request, into an SVG
file ahead of time: it routes the lines around the cards, animates the
flows and embeds its font, so the file looks the same everywhere. For a
chart, or a kind archgram does not draw, Mermaid is the better tool.

### How is it different from draw.io or Excalidraw?

draw.io and Excalidraw are canvases you arrange by hand, which suits a
sketch. archgram places everything from the spec, so the diagram lives in
git beside the code and changes in a diff when the system does.

### How is it different from D2 or PlantUML?

D2 is a language for many kinds of diagram, and PlantUML draws UML from
text, such as sequence, class and deployment diagrams. archgram draws two
kinds, the architecture of a system and the sequence of one request, from
a spec in plain YAML or JSON that an agent can write from the code; it
routes the lines, animates the flows inside the SVG and gives the same
file on every machine. For the rest of UML, PlantUML is the better tool;
for a diagram in many shapes, D2 is.

### How is it different from Structurizr and the C4 model?

Structurizr models a system once in its DSL and draws several C4 views
from that model; it is the C4 model's reference implementation. archgram
draws one diagram per spec, the parts and the path a request takes, as an
animated SVG for a README or a docs page. To keep several C4 views in
step, Structurizr is the better tool.

### Can a coding agent draw my architecture from the code?

Yes, with the `draw-architecture-diagram` skill (above): it reads the code, draws a part
only where a file backs it and an edge only where a line of code makes it,
and lists both.

### How do I keep architecture diagrams in sync with the code?

Keep the spec in git beside the code. archgram redraws the diagram from
it, and its agent skill can write the spec again from the code, so the
diagram changes in the same pull request as the system. When the spec
names the code behind each part and line (`source`), or the document
that states them, `archgram check` says which of it is gone, so CI can
fail a pull request that leaves the diagram behind.

### Can it use my design system's colours?

Yes: `--theme-file` maps archgram's colour roles to your design tokens
(W3C Design Tokens), and refuses colours that fall short of contrast
([docs/SPEC.md, Theme
file](https://github.com/byAbas/archgram/blob/main/docs/SPEC.md)).

### Will the same spec give the same file tomorrow?

Yes, byte for byte, on every machine, so a diagram changes in git only
when the system does.

## Documentation

- [archgram.dev](https://archgram.dev): what archgram does, with drawings it made.
- [docs/SPEC.md](https://github.com/byAbas/archgram/blob/main/docs/SPEC.md): the spec, every field and rule, with examples.
- [DESIGN.md](https://github.com/byAbas/archgram/blob/main/DESIGN.md): the visual rules.
- [ARCHITECTURE.md](https://github.com/byAbas/archgram/blob/main/ARCHITECTURE.md): how archgram works inside.
- [examples/](https://github.com/byAbas/archgram/tree/main/examples): complete specs.

## License

[MIT](https://github.com/byAbas/archgram/blob/main/LICENSE). The embedded font, Geist, is under the SIL Open Font
License; technology logos come from Simple Icons, CC0, except those that
carry a license of their own, each named in
`crates/archgram-icons/data/provenance.tsv`.
