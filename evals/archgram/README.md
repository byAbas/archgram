# Evaluations of the archgram skill

Twelve cases for `skills/archgram`, in the format of Anthropic's
[skill-creator](https://github.com/anthropics/skills/tree/main/skills/skill-creator),
the one the Agent Skills format's own guide to evaluating skills uses:
`evals.json` holds each prompt, what a good run produces and the checks a
grader marks it by (`expectations`), and `files/` the small projects the
prompts run in.

| Case | What it checks |
|---|---|
| `draws-a-project` | It draws a project from its code, every part backed by a file and every edge by a line, no wider than 1,300 px with the direction archgram chose written in the spec, in archgram's colours since none were asked for, without starting a browser |
| `asks-when-unclear` | It asks when the README and the code disagree, rather than drawing what does not exist |
| `updates-an-existing-diagram` | It changes an existing spec, keeping its direction, and says what changed |
| `not-for-other-work` | It stays out of a request that is not a diagram, and the work asked for is done |
| `draws-a-pipeline` | It recognises a data pipeline and follows one day's rows from their sources to the warehouse, with where bad rows go |
| `draws-ports-and-adapters` | It recognises ports and adapters and shows the rules depending on nothing outside their ports |
| `merges-parts-with-the-same-relations` | It merges parts no question tells apart and keeps a crowded system within about 10 nodes and 12 edges |
| `repairs-what-lost-its-code` | It starts from what `archgram check` says has lost its code, removes only that, keeps every source the check found and the spec's direction |
| `draws-in-the-project-colours` | Asked for the project's colours, it reads them from its CSS, writes them as tokens and a mapping that `archgram theme check` passes, and draws with them |
| `draws-a-sequence` | Asked what happens, step by step, when someone signs in, it draws a sequence: the participants in the order the request reaches them, the messages in the order the code runs, each call sourced to its line, the password's branch an `alt` with a refused 401, a send that does not wait, and phases |
| `says-a-kind-it-cannot-draw` | Asked for a state diagram, it says archgram does not draw one yet and offers what it can, rather than drawing the states as another kind |
| `draws-from-a-document` | Named a document and no code, it draws the example the document describes, each part and edge sourced to the sentence that states it, in the document's own words, with nothing the text does not state |

Every case that draws expects a source on each node a file backs and on
each edge, so it needs an archgram that reads them (0.7 or later;
docs/features/sources.md). `draws-a-sequence` needs one that draws
sequences with phases, 0.10 or later (docs/features/sequence.md).

Each case runs with the skill and without it, as skill-creator does. The
runs call a model and cost money, so they run on request, not in CI; the
skill runs the archgram version it names (`archgram@X.Y.Z`) through npx,
so that version must be on npm, and the runs need network access to the
npm registry.

## Running them

`run.sh` runs the cases in fresh `claude -p` sessions, each in its own
copy of the project, made a git repository of its own outside this one,
so neither this repository nor its skills folder is read. It says how many
runs it will start, and starts them only with `--yes`:

```sh
sh evals/archgram/run.sh --out ~/archgram-evals/iteration-1 --dry-run
sh evals/archgram/run.sh --out ~/archgram-evals/iteration-1 --repeat 3 --jobs 4 --yes
```

- **With the skill:** the skill as it is in `skills/archgram`, copied
  first, loads with `--plugin-dir`, as a plugin from Anthropic's
  directory does. A project's `.claude/skills` plugin does not load in a
  `-p` run
  ([what runs before you trust a folder](https://code.claude.com/docs/en/permissions)).
- **Without it:** the same prompt and project, with no skill.
- **Nothing else:** only the project's own settings load
  (`--setting-sources project`), and every plugin the user has on is
  turned off for the runs, so neither side gets help the other does not.
- **Unattended:** reading, writing in the copy and archgram at the
  skill's version run without a prompt; anything else is denied
  (`--permission-prompts none`), which also takes away the tool that asks
  the user a question, so a question is asked in the run's reply. The
  system's viewer is denied too, so a case expects the skill to try to
  open the drawing, not to open it. Each run
  stops at `--budget` dollars (5 by default) and 80 turns.
- **One runner per folder:** a second `run.sh` on the same `--out` is
  refused while the first runs, and a run already in the folder is never
  started again.

Before a release, run them against the archgram this repository builds,
in place of the version on npm, with `--archgram`:

```sh
cargo build --release -p archgram-cli
sh evals/archgram/run.sh --out ~/archgram-evals/pre-release --archgram target/release/archgram --yes
```

Each copy then holds that build as `node_modules/archgram` at the
skill's version, kept out of the copy's git: npx takes a package of the
version it asks for from the project's own `node_modules` before the
registry, so the skill's commands run unchanged. `run.json` names the
build.

Follow them live, in a second terminal, one line for each thing a run
does: its start, with the plugins and skill it loaded, what the model
says, each tool it calls, each error and denial, and how it ended, with
its time, turns and estimated cost. `--only eval-8` keeps one case,
`--full` prints the model's text whole, and `--once` prints what is there
and stops:

```sh
node evals/archgram/watch.mjs ~/archgram-evals/iteration-1
```

Each run leaves skill-creator's layout under `--out`:
`eval-<id>-<name>/<config>/run-<k>/` with `outputs/` (the project as the
run left it), `stream.jsonl` (each event as it happened, which the
watcher reads), `result.json`, `timing.json` (time, tokens and the
estimated cost), `transcript.jsonl` (the session's whole transcript) and
`stderr.log`. skill-creator's
grader then marks each run against its case's `expectations`, and its
`aggregate_benchmark` and viewer read the same folders.

## Third-party material

`files/bff-pattern/docs/patterns/backends-for-frontends.md` is
"Backends for Frontends pattern" from the Azure Architecture Center on
Microsoft Learn, by Microsoft and its contributors, copied unchanged from
[MicrosoftDocs/architecture-center](https://github.com/MicrosoftDocs/architecture-center/blob/130a96c829e7/docs/patterns/backends-for-frontends.md)
at commit `130a96c829e7`. It is licensed under the
[Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/)
(the repository's [LICENSE](https://github.com/MicrosoftDocs/architecture-center/blob/main/LICENSE)),
not under archgram's MIT license. Only text whose license allows it is
kept here.

