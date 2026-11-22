# Feature: the facts behind a diagram, found by kind before the spec

| Field   | Value      |
|---------|------------|
| Version | 0.7        |
| Date    | 2026-11-22 |
| Status  | Draft      |
| Release | Planned    |
| Issue   | [#165](https://github.com/byAbas/archgram/issues/165) |

> **Superseded (PRD 0.45).** The skills write no facts document and run no sizing script: each reads only what the diagram needs, and the spec's sources are the record of what backs it (docs/PRD.md, §6.6). What follows is kept as the record of an earlier design; nothing in it is a requirement.

What this feature does and why. Nothing else points here yet: whether
`docs/PRD.md` changes, and how, is decided once this draft is (Open
questions). How the skill runs it goes into `skills/archgram/` as it is
built, and the command it adds into `docs/SPEC.md` and `README.md`.

## Problem

The `archgram` skill draws a project from its code, and the drawing is
only as right as the facts the agent found. Today those facts are never
written down. The agent keeps them while it works, writes the spec from
them, draws it, and only then lists them in its report
(`skills/archgram/SKILL.md`, steps 2 and 7). A wrong fact reaches the
spec before anyone can see it.

`archgram check` holds every source in a spec to the code, so a part or
a line the agent made up is caught. A part it never found is not: the
check reads only what the spec names. On a large project, that is where
the drawing goes wrong.

The skill also reads only what one diagram needs, from the entry points
the reader cares about, and keeps no record of what it did not read.
The next diagram of the same project, of another kind, finds the same
parts again.

More kinds of diagram are coming (`docs/features/diagram-kinds.md`), and
each needs other facts: the order of the calls for a sequence, the
states and transitions for a lifecycle, the data and its stores for a
dataflow. The kind decides what to look for, so it has to come first.

### Evidence

- The skill's evaluation projects are small: the largest,
  `evals/archgram/files/bff-pattern`, is 2 files and 172 lines;
  `billing` is 12 files and 94 lines (counted 2026-11-19 with `find` and
  `wc`). No project the size of a real service, let alone Kubernetes,
  has been measured.
- A source's words are looked for anywhere in its file
  (`docs/SPEC.md`, Sources), so a line can pass the check when the words
  stand on another line than the one that makes the edge.
- Anthropic's guide to context for agents describes "context rot": as
  the tokens in context grow, the model recalls what is in it less
  accurately, gradually rather than at a cliff. It recommends sub-agents
  that work in a clean context and return a condensed summary, often
  1,000 to 2,000 tokens
  (https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents).
- In Anthropic's research system, several agents outperformed one by
  90.2 % on an internal research evaluation, using about 15 times the
  tokens of a chat; the gain was largest for broad work that does not fit
  one context, and smallest where every agent needs the same context
  (https://www.anthropic.com/engineering/multi-agent-research-system).
  Reading a codebase is the first kind; writing one spec is the second.
- Kubernetes shows a fact a scan can get wrong: its components never
  call each other directly; "All API usage from nodes (or the pods they
  run) terminates at the API server"
  (https://kubernetes.io/docs/concepts/architecture/control-plane-node-communication/).
  A scan that guesses from names would draw the scheduler calling the
  kubelet.

## Who it serves

| User | What they get |
|---|---|
| The user of a skill | One prompt draws the diagram; the facts behind it are in one document they may read, and nothing waits for them |
| The reader of the diagram | Every part and line backed by the code that makes it, and a list of what was not read |
| The maintainer of a project | One document of the project's parts that every later diagram, of any kind, starts from |
| The author of a new kind | A pack to write, not a change to the skill's core |

## Requirements

### The flow

The user calls the skill of the kind they want (PRD §6.6). It runs these
steps in order, in one context, and stops for no one:

1. **Size.** `archgram-size` measures what it would read, and it reads
   area by area.
2. **Find.** It reads the code as its kind's pack says, opening every
   line it cites.
3. **Document.** It writes the facts into `docs/diagrams/facts.md`.
4. **Spec.** It writes the spec from the document and checks it against
   the code with `archgram check`.

### Kind packs

- Each kind is a folder, `references/kinds/<kind>/`, of two files, so an
  agent that only finds facts reads only the first:
  - `find.md`: which facts the kind needs and in what form, where the
    code holds them, and what is expected;
  - `spec.md`: how the facts become the kind's spec, how much of them to
    draw, what an update keeps, and what the report adds.

  Each kind's pack is now that kind's own skill (PRD §6.6).
- A pack may say what counts as evidence for its kind. A workflow's
  roles, for one, are more often written in a document than in code.
- Adding a kind adds a pack and changes nothing in the core.

### The document

- A core layer that every kind shares: the parts with their files and
  technology, the boundaries, the entry points, and what was read and
  what was not, with why.
- A layer per kind, scanned only when that kind is asked for.
- Markdown tables, which the user reads and the spec step reads.
- Every relation carries its file, its line and a few words of that
  line, which become the spec's `source` as they are. Sources point at
  the code, never at the document, so `archgram check` keeps holding the
  drawing to the code.
- Every fact carries its evidence:

  | Level | Evidence | Reaches the spec |
  |---|---|---|
  | 1 | A line of code that makes it: a call, a read, a write, a message sent | Yes |
  | 2 | A configuration or manifest that wires it: a compose file, a chart, a DI container | Yes, with its source |
  | 3 | A name, a folder's convention, a generated file | No, an open question |
  | 4 | Only a README or a comment says so | No, an open question |

- What the pack expects and the scan did not find is listed as
  "expected, not found".

### What is not read

Three layers, each catching what the one before leaves:

1. Only the files git tracks.
2. The project's own marks in `.gitattributes`, `linguist-vendored` and
   `linguist-generated`
   (https://github.com/github-linguist/linguist/blob/main/docs/overrides.md).
3. A list of its own:
   - always left out, tracked or not: `vendor/`, `third_party/`,
     `node_modules/`, `bower_components/`, `Pods/`, `.venv/`, `venv/`,
     `__pycache__/`, `*.min.js`, `*.map` and lock files;
   - generated files, by their header (Go's `Code generated ... DO NOT
     EDIT.`, https://pkg.go.dev/cmd/go#hdr-Generate_Go_files_by_processing_source)
     or their name (`zz_generated*`, `*.pb.go`, `*_pb2.py`);
   - left out only when git ignores them or they are generated: `build/`,
     `out/`, `dist/`, `target/`, `bin/`, which are sometimes source.

Everything left out is listed in the document's coverage, with how many
files and why. A generated file is not read, but noted: a generated
client is a hint, level 3, and the scan looks for the call in the code
that is not generated.

### The engine

- `archgram spec --kinds` prints, one line each, every kind this
  archgram draws: the value `diagram:` takes, the `--section` that
  describes its format, and the question it answers. It prints only the
  kinds it draws, not the planned ones.
- The list is built from `DiagramKind` (`crates/archgram-core/src/spec.rs`),
  so a new kind cannot be drawn without being listed. A test holds every
  listed kind to a section of `docs/SPEC.md`.
- A kind's skill draws only when `spec --kinds` lists its kind; when
  the project's archgram does not, the skill says it is too old for
  that kind, and draws nothing.

## Limits

- What the skill did not read stays unknown, and the coverage says where.
- Each skill reads in one context, so a large project is read area by
  area, each area's facts written down before the next. Its accuracy on
  a large project is not measured.
- The time a run takes is not measured. An estimate, from the current
  skill's 56 seconds on small projects (`docs/PRD.md`, Problem): about 3
  to 6 minutes for a small project, 6 to 12 for a medium one, 10 to 25
  for one the size of Kubernetes. archgram's own `build` takes about
  0.1 s on the examples; the rest is the agent.

## Not in this

- Kinds archgram does not draw. A pack follows its kind into the engine.
- Documentation for its own sake. The document is the facts for
  diagrams, not a guide to the project.
- Reading a document the user names instead of the code. The skill does
  that today, and this changes nothing there.
- Running the scan in CI.

## Success criteria

- **Kubernetes**, at the containers level, from a shallow clone: the
  document names every control plane and node component the official
  page lists (https://kubernetes.io/docs/concepts/overview/components/),
  draws no edge that "terminates at the API server" contradicts, and
  every source points at a file that is there.
- The same repository, scanned twice, leaves out the same files and
  reports the same coverage.
- A new evaluation project of medium size, large enough that `archgram-size`
  splits it, with a `vendor/` folder, a generated file and a line that
  only looks like a call: the skill leaves out the first two, and drops
  the third.
- The skill's twelve evaluation cases pass as often as before.
- A pack for a third kind, once the engine draws one, changes no file
  of the core.

## Decided (2026-11-19)

1. **One skill for each kind** (PRD 0.45, replacing one skill that
   took the kind first). No sub-agents: the scanner and verifier agents
   first planned here were never built, and each skill reads in one
   context.
2. *Replaced by 1.*
3. *Replaced by The flow above:* size, find, document, spec.
4. **A pack for each kind,** now the kind's own skill.
5. **A core layer and a layer per kind** in one document, in Markdown
   tables.
6. **Evidence levels,** and only levels 1 and 2 reach the spec, its
   sources pointing at the code.
7. **Three layers of what is not read,** each listed in the coverage.
8. **Size before reading,** reported, never asked about.
9. **`archgram spec --kinds`,** built from `DiagramKind`; the skill
   reads the kinds from it.
10. **One document for the project, `docs/diagrams/facts.md`,** beside
    the specs: its core once, and a layer for each diagram drawn from
    it, `## <Kind>: <name>`, so a second diagram starts from the parts
    the first found. A document per diagram would repeat the core.
11. **No stop for approval** (PRD 0.45, replacing a stop before the
    spec). Nothing stamps the document; where the code and a
    document disagree, the code is drawn and the report says so. Only a
    request to see the facts first makes the skill stop.
12. **Sizing is a script in the skill, `scripts/size.mjs`,** run with
    Node before the code is read. It takes git's list of tracked files,
    the project's `.gitattributes` through `git check-attr`, and the
    rest of `references/ignore.md`; it counts the lines and the bytes of
    what is left, about four bytes a token, and splits it into areas no
    larger than one reader's budget, 100,000 tokens until measured. It
    reads only under the project's folder, follows no symbolic link,
    prints no file's contents and uses no network. Its test,
    `scripts/skill-size.test.mjs`, runs in CI on the Node versions the
    launcher is tested on. Since 2026-11-20 it ships in the `archgram`
    npm package as `archgram-size`, `packages/archgram/skill/size.mjs`,
    no longer in the skill's folder: Anthropic's directory accepts only
    text and images in a plugin from an organization it has not Verified
    (#176).

## Open questions

- The limits between a small, a medium and a large scope, to be
  measured, not guessed. A first guess, to replace: about 100,000 tokens
  for one reader.
- How a user changes what is not read: a file in the project, or their
  request.
- How the skill knows the core layer has gone stale between two
  diagrams, beyond what `archgram check` says of each spec's sources.

## Changelog

| Version | Date       | Change |
|---------|------------|--------|
| 0.1     | 2026-11-19 | First draft, from #165: one skill that takes the kind first, measures the scope, scans it with read-only sub-agents in clean contexts, verifies every cited line and writes the facts as a document the user approves before the spec; a core layer and a layer per kind, with evidence levels; what is not read, in three layers; `archgram spec --kinds`, built from `DiagramKind`. Open: the verifier, the models, where sizing lives, its limits, the document's place, the PRD. |
| 0.2     | 2026-11-19 | The document is one for the project, `docs/diagrams/facts.md`, its core once and a layer per diagram; the skill stops for the user's approval of it before a spec, unless the request says to go on (Decided, 10 and 11). The PRD points here (§6.6). |
| 0.3     | 2026-11-19 | Sizing is a script in the skill, `scripts/size.mjs`, tested in CI; whether it moves into archgram stays open (Decided, 12). |
| 0.4     | 2026-11-20 | The sizing script ships in the `archgram` npm package as `archgram-size`, outside the skill's folder, and stays JavaScript rather than moving into the archgram binary; the open question is closed (Decided, 12). |
| 0.5     | 2026-11-22 | One skill for each kind, reading in one context: the never-built sub-agents are gone from this document; the facts are stamped without a stop, a disagreement reported rather than asked (PRD 0.45; Decided, 1, 3, 8, 11). |
| 0.6     | 2026-11-22 | No stamp: `archgram-facts` is removed, and the spec is written from the document (approved-drawing.md, Decided, 8). |
| 0.7     | 2026-11-22 | Superseded: no facts document and no `archgram-size`; each skill reads only what the diagram needs, and the spec's sources are the record (PRD 0.45, §6.6). |
