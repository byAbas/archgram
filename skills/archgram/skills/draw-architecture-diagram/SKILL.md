---
name: draw-architecture-diagram
description: Draws a project's software architecture diagram, what it is made of and what calls what, as an animated SVG with archgram, from its code or from a document the user names. Writes a spec in docs/diagrams/, checks it against the code and draws it, then reports briefly how exact it is. Use it when the user asks for an architecture diagram, or to update or repair one archgram drew.
license: MIT
compatibility: Requires Node 22 or later, with npx.
---

# Draw an architecture diagram

archgram lays out the boxes, routes the lines and draws light and dark in
one SVG. Your work: understand the project, write a short spec, and back
every box and line with the code. The rules every kind shares come from
`archgram guide` (step 1); `find.md` beside this file says what an
architecture draws.

## Find archgram

Every command runs archgram through npx (Node 22 or later), found where
it already is, once, before the first command. Read the project's root
`package.json` first.

- **The project's own.** When `package.json` lists archgram, run that
  one: write each command with `archgram` for `archgram@0.13.0` and
  `--no --offline` for `--yes`. When it is older than `0.13.0`, say so,
  with the project's own command to upgrade (`npm install --save-dev
  archgram@0.13.0`), and stop. If npx says it is missing, ask the user
  to install the project's dependencies, and stop.
- **This version, already on the machine:**

  ```bash
  npx --no --offline --loglevel=error archgram@0.13.0 --version
  ```

  It uses no network. When it prints `archgram 0.13.0`, run the commands
  as written.
- **Not there: ask before downloading.** Say what and from where: the npm
  package `archgram@0.13.0` and its native program, a few megabytes, from
  the npm registry. Offer to install it into the project (`npm install
  --save-dev archgram@0.13.0`) or to let npx fetch it once, and run
  neither until the user picks one.

## 1. Read the rules

```bash
npx --yes --loglevel=error archgram@0.13.0 guide
```

Then read `find.md`.

## 2. Understand the project

Read only what the architecture needs, as `find.md` says: never the
whole codebase.

## 3. Write the spec, then check it

```bash
npx --yes --loglevel=error archgram@0.13.0 spec --brief
```

Write `docs/diagrams/architecture.archgram.yaml` (another name when the
user asks for one), then check it until it passes:

```bash
npx --yes --loglevel=error archgram@0.13.0 check docs/diagrams/architecture.archgram.yaml
```

On an update, start from the spec there: `check` names what lost its
code; change only that and what the request asks.

## 4. Draw, look, open, report

```bash
npx --yes --loglevel=error archgram@0.13.0 build docs/diagrams/architecture.archgram.yaml --preview
```

Hold it to the guide's limits, look at the previews, write the direction
`build` chose into the spec, open the drawing and report, as the guide
says.
