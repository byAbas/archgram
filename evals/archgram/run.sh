#!/bin/sh
# Runs the archgram skill's evaluations (README.md here): each case of
# evals.json in a copy of its project, with the skill and without it, in a
# fresh `claude -p` session. Every run is a paid model call, so the script
# says how many runs it will start and starts none without --yes.
#
#   sh evals/archgram/run.sh --out DIR [--cases 1,3] [--configs with_skill]
#     [--repeat N] [--model M] [--jobs N] [--budget USD] [--archgram BIN]
#     [--dry-run] [--yes]
#
# It writes skill-creator's layout under DIR, a folder outside the
# repository: eval-<id>-<name>/eval_metadata.json, and for each run
# eval-<id>-<name>/<config>/run-<k>/ with outputs/ (the project as the run
# left it), stream.jsonl (each event as it happened; follow the runs live
# with `node evals/archgram/watch.mjs DIR`), result.json, timing.json,
# transcript.jsonl and stderr.log.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)

die() {
  printf 'run.sh: %s\n' "$*" >&2
  exit 1
}

# One run, started by the plan below through xargs; the options arrive in
# the environment.
if [ "${1:-}" = __one ]; then
  id=$2 config=$3 k=$4
  case_dir=$(node -e '
    const c = require(process.argv[1]).evals.find((e) => e.id === Number(process.argv[2]));
    console.log(`eval-${c.id}-${c.name}`);' "$here/evals.json" "$id")
  dir="$EVAL_OUT/$case_dir/$config/run-$k"
  # mkdir, without -p, fails when the folder exists: one step, so two
  # runners sharing an --out never start the same run.
  mkdir -p "$(dirname "$dir")"
  mkdir "$dir" 2>/dev/null || die "$dir exists; give a new --out"
  files=$(node -e '
    const c = require(process.argv[1]).evals.find((e) => e.id === Number(process.argv[2]));
    console.log(c.files[0]);' "$here/evals.json" "$id")
  cp -R "$here/$files" "$dir/outputs"
  # With --archgram, the skill's `npx … archgram@X.Y.Z` runs that build:
  # npx takes a package of the version it asks for from the project's own
  # node_modules before the registry. Kept out of the copy's git.
  if [ -n "${EVAL_ARCHGRAM:-}" ]; then
    pkg="$dir/outputs/node_modules/archgram"
    mkdir -p "$pkg/bin" "$dir/outputs/node_modules/.bin"
    printf '{ "name": "archgram", "version": "%s", "bin": { "archgram": "bin/archgram.js" } }\n' \
      "$EVAL_VERSION" >"$pkg/package.json"
    printf '#!/usr/bin/env node\nconst r = require("child_process").spawnSync(%s, process.argv.slice(2), { stdio: "inherit" });\nprocess.exit(r.status ?? 1);\n' \
      "$(node -e 'console.log(JSON.stringify(process.argv[1]))' "$EVAL_ARCHGRAM")" >"$pkg/bin/archgram.js"
    chmod +x "$pkg/bin/archgram.js"
    ln -s ../archgram/bin/archgram.js "$dir/outputs/node_modules/.bin/archgram"
    printf 'node_modules/\n' >>"$dir/outputs/.git-exclude"
  fi
  # Its own git repository, so archgram's project folder is the copy and
  # nothing of this repository is read.
  (cd "$dir/outputs" && git init -q &&
    { [ ! -f .git-exclude ] || { cat .git-exclude >>.git/info/exclude && rm .git-exclude; }; } &&
    git add -A &&
    git -c user.name=archgram-eval -c user.email=eval@archgram.invalid \
      commit -qm "the project as given")

  # Read-only commands and archgram at the skill's version run without a
  # prompt; anything else is denied (--permission-prompts none), so a run
  # never waits and never changes anything outside its copy.
  set -- --model "$EVAL_MODEL" --setting-sources project \
    --settings "$EVAL_OUT/settings.json" \
    --permission-mode acceptEdits --permission-prompts none \
    --max-budget-usd "$EVAL_BUDGET" --max-turns 80 \
    --output-format stream-json --verbose \
    --allowedTools Read Write Edit Glob Grep Skill \
    "Bash(npx --yes --loglevel=error archgram@$EVAL_VERSION *)" \
    "Bash(ls *)" "Bash(cat *)" "Bash(head *)" "Bash(grep *)" "Bash(find *)" \
    "Bash(wc *)" "Bash(git ls-files*)" "Bash(git status*)" "Bash(git diff*)" \
    "Bash(node --version)"
  # With the skill, as the directory's plugin, for this session only: a
  # project's .claude/skills plugin does not load in a -p run.
  [ "$config" = without_skill ] ||
    set -- "$@" --plugin-dir "$EVAL_OUT/skill-snapshot/archgram"

  prompt=$(node -e '
    console.log(require(process.argv[1]).evals.find((e) => e.id === Number(process.argv[2])).prompt);' \
    "$here/evals.json" "$id")
  start=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  start_s=$(date +%s)
  printf '%s %s run-%s: started\n' "$case_dir" "$config" "$k"
  status=0
  # Each event as it happens, which watch.mjs follows live; the last line
  # is the run's result.
  (cd "$dir/outputs" && printf '%s' "$prompt" | claude -p "$@") \
    >"$dir/stream.jsonl" 2>"$dir/stderr.log" || status=$?
  end=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  end_s=$(date +%s)

  node -e '
    const fs = require("fs"), path = require("path"), os = require("os");
    const [dir, start, end, seconds, status] = process.argv.slice(1);
    let r = {};
    for (const line of fs.readFileSync(path.join(dir, "stream.jsonl"), "utf8").split("\n")) {
      try { const e = JSON.parse(line); if (e.type === "result") r = e; } catch {}
    }
    fs.writeFileSync(path.join(dir, "result.json"), JSON.stringify(r, null, 2) + "\n");
    const u = r.usage || {};
    const tokens = ["input_tokens", "output_tokens", "cache_read_input_tokens", "cache_creation_input_tokens"]
      .reduce((n, k) => n + (u[k] || 0), 0);
    fs.writeFileSync(path.join(dir, "timing.json"), JSON.stringify({
      total_tokens: tokens,
      duration_ms: r.duration_ms ?? Number(seconds) * 1000,
      total_duration_seconds: (r.duration_ms ?? Number(seconds) * 1000) / 1000,
      executor_start: start, executor_end: end,
      executor_duration_seconds: Number(seconds),
      total_cost_usd: r.total_cost_usd ?? null,
      num_turns: r.num_turns ?? null,
      exit_status: Number(status),
    }, null, 2) + "\n");
    // The session transcript, for grading what the run did, not only what
    // it left.
    if (r.session_id) {
      const projects = path.join(os.homedir(), ".claude", "projects");
      for (const p of fs.existsSync(projects) ? fs.readdirSync(projects) : []) {
        const f = path.join(projects, p, r.session_id + ".jsonl");
        if (fs.existsSync(f)) { fs.copyFileSync(f, path.join(dir, "transcript.jsonl")); break; }
      }
    }' "$dir" "$start" "$end" "$((end_s - start_s))" "$status"
  printf '%s %s run-%s: exit %s, %ss\n' "$case_dir" "$config" "$k" "$status" "$((end_s - start_s))"
  exit 0
fi

out= cases= configs="with_skill without_skill" repeat=1 archgram=
model=claude-opus-5-5 jobs=1 budget=5 dry=false yes=false
while [ $# -gt 0 ]; do
  case $1 in
    --out) out=$2; shift 2 ;;
    --cases) cases=$(printf '%s' "$2" | tr ',' ' '); shift 2 ;;
    --configs) configs=$(printf '%s' "$2" | tr ',' ' '); shift 2 ;;
    --repeat) repeat=$2; shift 2 ;;
    --model) model=$2; shift 2 ;;
    --jobs) jobs=$2; shift 2 ;;
    --budget) budget=$2; shift 2 ;;
    --archgram) archgram=$2; shift 2 ;;
    --dry-run) dry=true; shift ;;
    --yes) yes=true; shift ;;
    *) sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//' >&2; exit 2 ;;
  esac
done
[ -n "$out" ] || die "--out DIR is required"
case $out in /*) ;; *) out="$PWD/$out" ;; esac
case $out in "$root"/*|"$root") die "--out must be outside the repository: $out" ;; esac
for c in $configs; do
  case $c in with_skill|without_skill) ;; *) die "a config is with_skill or without_skill, not $c" ;; esac
done
if [ -n "$archgram" ]; then
  case $archgram in /*) ;; *) archgram="$PWD/$archgram" ;; esac
  [ -x "$archgram" ] || die "--archgram $archgram is not a program"
fi
command -v claude >/dev/null || die "claude is not on the PATH"
command -v node >/dev/null || die "node is not on the PATH"
[ -n "$cases" ] || cases=$(node -e 'console.log(require(process.argv[1]).evals.map((e) => e.id).join(" "))' "$here/evals.json")

version=$(grep -o 'archgram@[0-9][0-9.]*[0-9]' "$root/skills/archgram/SKILL.md" | head -1 | cut -d@ -f2)
plan=$(for id in $cases; do for c in $configs; do
  k=1; while [ "$k" -le "$repeat" ]; do echo "$id $c $k"; k=$((k + 1)); done
done; done)
n=$(printf '%s\n' "$plan" | grep -c .)
printf 'run.sh: %s runs: cases %s, %s, %s each, model %s, at most $%s each, archgram@%s%s\n' \
  "$n" "$(echo $cases | tr ' ' ',')" "$(echo $configs | tr ' ' ',')" "$repeat" "$model" "$budget" "$version" \
  "${archgram:+ (run by $archgram)}"
if $dry; then
  printf '%s\n' "$plan"
  exit 0
fi
$yes || die "each run is a paid model call; read the line above and run again with --yes"

mkdir -p "$out"
out=$(cd "$out" && pwd)
# One runner per folder: a second one, started by mistake, would share
# the snapshot and the settings and race the first for each run.
mkdir "$out/.running" 2>/dev/null ||
  die "another run.sh is running in $out; if none is, one stopped early: remove $out/.running"
trap 'rmdir "$out/.running"' EXIT
# The skill as it is now, so the runs test one version even if the
# working tree changes while they run.
[ -e "$out/skill-snapshot" ] || { mkdir -p "$out/skill-snapshot" && cp -R "$root/skills/archgram" "$out/skill-snapshot/archgram"; }
# Every plugin the user has on is off for the runs, so neither
# configuration gets help the other does not.
claude plugin list --json | node -e '
  let s = ""; process.stdin.on("data", (d) => s += d).on("end", () => {
    const off = Object.fromEntries(JSON.parse(s).filter((p) => p.enabled).map((p) => [p.id, false]));
    console.log(JSON.stringify({ enabledPlugins: off }));
  });' >"$out/settings.json"
node -e '
  const fs = require("fs"), path = require("path");
  const [evals, out, cases, commit, model, version, build] = process.argv.slice(1);
  for (const id of cases.split(" ")) {
    const c = require(evals).evals.find((e) => e.id === Number(id));
    const dir = path.join(out, `eval-${c.id}-${c.name}`);
    fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(path.join(dir, "eval_metadata.json"), JSON.stringify(
      { eval_id: c.id, eval_name: c.name, prompt: c.prompt, assertions: c.expectations }, null, 2) + "\n");
  }
  fs.writeFileSync(path.join(out, "run.json"), JSON.stringify(
    { commit, model, archgram: version, build: build || null, started: new Date().toISOString() }, null, 2) + "\n");' \
  "$here/evals.json" "$out" "$cases" "$(git -C "$root" rev-parse HEAD)" "$model" "$version" "$archgram"

export EVAL_OUT="$out" EVAL_MODEL="$model" EVAL_BUDGET="$budget" EVAL_VERSION="$version" \
  EVAL_ARCHGRAM="$archgram"
printf '%s\n' "$plan" | xargs -P "$jobs" -L 1 sh "$0" __one
