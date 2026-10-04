# archgram

A skill, and a Claude Code plugin of that one skill, that draws a
project's software architecture as an animated SVG for its README and
docs. The agent reads the code and the documentation, writes a spec to
`docs/diagrams/<name>.archgram.yaml`, and archgram lays it out and draws
`docs/diagrams/<name>.svg` beside it, in light and dark, in the project's
own colours. The report lists each part drawn with the file behind it.

## What it runs, reads and writes

- It runs archgram through npx: the project's own archgram when its
  `package.json` lists one, and otherwise `archgram@0.6.1`, the version
  released with this skill. npx downloads that version from the npm
  registry the first time.
- It reads the project's code, documentation and styling files.
- It writes the spec and the SVG under `docs/diagrams/`. When the project
  has colours of its own, it also writes `archgram.theme.json` at the
  project's root and, where the colours are not already design tokens,
  `docs/diagrams/theme/archgram.resolver.json`.
- It opens the SVG once with the system's own viewer.
- It sends nothing anywhere, and edits the README or commits only when
  asked.

## More

Instructions: [SKILL.md](SKILL.md). The command, its install and the spec:
[the archgram README](https://github.com/byAbas/archgram#readme) and
[archgram.dev](https://www.archgram.dev). Licence: MIT.
