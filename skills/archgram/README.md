# archgram

Two skills, and a Claude Code plugin of them in
[Anthropic's directory](https://claude.ai/directory), each drawing one
kind of diagram as an animated SVG for a project's README and docs:

| Skill | Ask | Draws |
|---|---|---|
| `/archgram:draw-architecture-diagram` | `Draw the architecture of this project.` | What the system is made of and what calls what |
| `/archgram:draw-sequence-diagram` | `Draw the sequence of signing in.` | The order in which the parts handle one request |

From one prompt, the agent reads the code and the documentation, writes a spec to
`docs/diagrams/<name>.archgram.yaml`, and archgram lays it out and draws
`docs/diagrams/<name>.svg` beside it, in light and dark, in archgram's
colours or, when asked, the project's own. Its report is short, about
fifteen lines: what it drew, how exact it is, what it changed after
looking and what it left out; each part's file and each line's code are
in the spec's sources.

## Install

In Claude Code, add it from [Anthropic's directory](https://claude.ai/directory),
or, without a claude.ai account, install it from its repository in a
session (Claude Code 2.1.275 or later):

```text
/plugin install archgram --marketplace byAbas/archgram
```

For other agents, and every other way, see
[the archgram README](https://github.com/byAbas/archgram#draw-with-your-coding-agent).

## What it runs, reads and writes

- It runs archgram through npx: the project's own archgram when its
  `package.json` lists one, and otherwise `archgram@0.13.1`, the version
  released with these skills, when the machine already has it. It looks
  for it with no network and installs nothing while it looks. When the
  project's own is older than `archgram@0.13.1`, it says so, with the
  command to upgrade it, and draws nothing; it upgrades only when you
  ask.
- It downloads nothing you have not agreed to, the one thing it asks
  about. When it finds no archgram, it stops and asks, naming the npm package `archgram@0.13.1`
  and the native archgram program for your system, a few megabytes, from
  the npm registry. You choose to install it into the project, pinned by
  its lockfile, or to let npx download it once, and it does neither
  until you answer.
- It reads only what the diagram needs, never the whole codebase: the
  file list (`git ls-files`, names only), each app's manifest, its run
  files and entry points, and the line behind each line it draws; its
  styling files only when asked for its colours. It never opens a file
  that commonly holds secrets (`.env`, `*.pem`, `.npmrc` and the like).
- From archgram 0.7, a spec may name the file behind each part and a few
  words of the line behind each edge, and `check` and `build` look for
  them, to say which of that code is gone. archgram reads those files
  only under the project's folder, never `.git` nor a file that commonly
  holds secrets, and keeps only whether each holds the words
  ([SECURITY.md, What archgram reads](https://github.com/byAbas/archgram/blob/main/SECURITY.md#what-archgram-reads)).
- It writes the spec only from what a line of code or a configuration
  backs. It does not stop for you to check what it found; ask it to show
  you first, and it will.
- The rules both skills share come from archgram itself, `archgram
  guide`, so they are always those of the archgram that draws.
- It writes the spec and the SVG under `docs/diagrams/`, and archgram
  writes the still previews it looks at under `.archgram/preview/`, a
  folder git leaves out. When the user
  asks for the project's colours, it also writes `archgram.theme.json`
  at the project's root and, where the colours are not already design
  tokens, `docs/diagrams/theme/archgram.resolver.json`.
- It opens the SVG once with the system's own viewer.
- It pre-approves no command. Every command it runs, `guide`, `spec`,
  `check`, `build` and `theme check`, the project's own archgram included, asks
  the user first, unless the user has already allowed it.
- It sends nothing anywhere, and edits the README or commits only when
  asked.

## Privacy

The skill and the archgram command collect no personal data, keep none
and send none: there is no account, no telemetry and no analytics.

- Everything runs on your machine. The skill reads the project's files
  and writes the diagram's files there, where you can change or delete
  them; the archgram command makes no network request.
- The one request goes to the npm registry, when you agree to download
  `archgram@0.13.1`; a project that lists archgram has installed it
  already. npm's
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

archgram, these skills included, is free and open source under the
[MIT licence](https://github.com/byAbas/archgram/blob/main/LICENSE): you
may use, copy, change and share it, and it comes as is, without warranty
of any kind. There is no account and nothing to pay. Downloading archgram
from the npm registry comes under npm's
[terms of use](https://docs.npmjs.com/policies/terms).

## More

Instructions: [draw-architecture-diagram](skills/draw-architecture-diagram/SKILL.md),
[draw-sequence-diagram](skills/draw-sequence-diagram/SKILL.md). The command, its install and the spec:
[the archgram README](https://github.com/byAbas/archgram#readme) and
[archgram.dev](https://www.archgram.dev).
