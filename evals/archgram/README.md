# Evaluations of the archgram skill

Nine cases for `skills/archgram`, in the format of Anthropic's
[skill-creator](https://github.com/anthropics/skills/tree/main/skills/skill-creator),
the one the Agent Skills format's own guide to evaluating skills uses:
`evals.json` holds each prompt, what a good run produces and the checks a
grader marks it by (`expectations`), and `files/` the small projects the
prompts run in.

| Case | What it checks |
|---|---|
| `draws-a-project` | It draws a project from its code, every part backed by a file and every edge by a line, no wider than 1,300 px, in archgram's colours since none were asked for, without starting a browser |
| `asks-when-unclear` | It asks when the README and the code disagree, rather than drawing what does not exist |
| `updates-an-existing-diagram` | It changes an existing spec and says what changed |
| `not-for-other-work` | It stays out of a request that is not a diagram, and the work asked for is done |
| `draws-a-pipeline` | It recognises a data pipeline and follows one day's rows from their sources to the warehouse, with where bad rows go |
| `draws-ports-and-adapters` | It recognises ports and adapters and shows the rules depending on nothing outside their ports |
| `merges-parts-with-the-same-relations` | It merges parts no question tells apart and keeps a crowded system within about 10 nodes and 12 edges |
| `repairs-what-lost-its-code` | It starts from what `archgram check` says has lost its code, removes only that, and keeps every source the check found |
| `draws-in-the-project-colours` | Asked for the project's colours, it reads them from its CSS, writes them as tokens and a mapping that `archgram theme check` passes, and draws with them |

Every case that draws expects a source on each node a file backs and on
each edge, so it needs an archgram that reads them (0.7 or later;
docs/features/sources.md).

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
  the user a question, so a question is asked in the run's reply. Each run
  stops at `--budget` dollars (5 by default) and 80 turns.
- **One runner per folder:** a second `run.sh` on the same `--out` is
  refused while the first runs, and a run already in the folder is never
  started again.

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
