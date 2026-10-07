# archgram

A skill, and a Claude Code plugin of that one skill in
[Anthropic's directory](https://claude.ai/directory), that draws a
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
- From archgram 0.7, a spec may name the file behind each part and a few
  words of the line behind each edge, and `check` and `build` look for
  them, to say which of that code is gone. archgram reads those files
  only under the project's folder, never `.git` nor a file that commonly
  holds secrets, and keeps only whether each holds the words
  ([SECURITY.md, What archgram reads](https://github.com/byAbas/archgram/blob/main/SECURITY.md#what-archgram-reads)).
- It writes the spec and the SVG under `docs/diagrams/`. When the project
  has colours of its own, it also writes `archgram.theme.json` at the
  project's root and, where the colours are not already design tokens,
  `docs/diagrams/theme/archgram.resolver.json`.
- It opens the SVG once with the system's own viewer.
- It pre-approves one command, the one that only prints the spec format,
  at that version: `npx --yes --loglevel=error archgram@0.6.1 spec`.
  Every other command, `check`, `build`, `theme check` and the project's
  own archgram included, asks the user first.
- It sends nothing anywhere, and edits the README or commits only when
  asked.

## Privacy

The skill and the archgram command collect no personal data, keep none
and send none: there is no account, no telemetry and no analytics.

- Everything runs on your machine. The skill reads the project's files
  and writes the diagram's files there, where you can change or delete
  them; the archgram command makes no network request.
- The one request goes to the npm registry, when npx downloads
  `archgram@0.6.1` the first time; a project that lists archgram has
  installed it already. npm's
  [privacy policy](https://docs.npmjs.com/policies/privacy) covers that
  request.
- What your coding agent reads to draw the diagram, the project's code
  and docs among it, is part of your conversation with that agent, and
  its provider's own terms and privacy policy cover it, as they cover
  any other request.

Questions go to [GitHub issues](https://github.com/byAbas/archgram/issues),
and vulnerabilities as
[SECURITY.md](https://github.com/byAbas/archgram/blob/main/SECURITY.md)
says.

## Terms

archgram, this skill included, is free and open source under the
[MIT licence](https://github.com/byAbas/archgram/blob/main/LICENSE): you
may use, copy, change and share it, and it comes as is, without warranty
of any kind. There is no account and nothing to pay. Downloading archgram
from the npm registry comes under npm's
[terms of use](https://docs.npmjs.com/policies/terms).

## More

Instructions: [SKILL.md](SKILL.md). The command, its install and the spec:
[the archgram README](https://github.com/byAbas/archgram#readme) and
[archgram.dev](https://www.archgram.dev).
