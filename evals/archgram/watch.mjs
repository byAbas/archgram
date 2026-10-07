// Follows the runs of run.sh as they happen: one line per thing a run
// does, read from each run's stream.jsonl (README.md here).
//
//   node evals/archgram/watch.mjs DIR [--only TEXT] [--full] [--once]
//
// --only shows the runs whose folder holds TEXT (`eval-8`, `without_skill`);
// --full prints the model's text whole rather than its first line; --once
// prints what is there and stops, rather than following.

import fs from "node:fs";
import path from "node:path";

const args = process.argv.slice(2);
const dir = args.find((a) => !a.startsWith("--") && args[args.indexOf(a) - 1] !== "--only");
const only = args.includes("--only") ? args[args.indexOf("--only") + 1] : "";
const full = args.includes("--full");
const once = args.includes("--once");
if (!dir) {
  console.error("usage: node evals/archgram/watch.mjs DIR [--only TEXT] [--full] [--once]");
  process.exit(2);
}

// eval-8-repairs-what-lost-its-code/with_skill/run-1 -> "8 with_skill 1".
const label = (run) => {
  const [c, config, k] = path.relative(dir, run).split(path.sep);
  return `${c.replace(/^eval-(\d+)-.*/, "$1")} ${config} ${k.replace("run-", "")}`;
};
const short = (s, n = 110) => {
  const one = String(s ?? "").trim().split("\n")[0];
  return one.length > n ? one.slice(0, n - 1) + "…" : one;
};
const what = (tool, input = {}) => {
  const rel = (p) => (p ? String(p).replace(/^.*\/outputs\//, "") : "");
  switch (tool) {
    case "Bash": return `Bash  ${short(input.command)}`;
    case "Read": case "Write": case "Edit": return `${tool.padEnd(5)} ${rel(input.file_path)}`;
    case "Glob": case "Grep": return `${tool.padEnd(5)} ${short(input.pattern)}`;
    case "Skill": return `Skill ${input.skill ?? input.command ?? ""}`;
    default: return `${tool} ${short(JSON.stringify(input), 80)}`;
  }
};

function describe(e) {
  if (e.type === "system" && e.subtype === "init") {
    const plugins = (e.plugins ?? []).map((p) => p.name ?? p).join(", ") || "none";
    const skills = (e.skills ?? []).filter((s) => /archgram/i.test(s)).join(", ") || "no archgram skill";
    return [`▶ started: ${e.model ?? ""}; plugins: ${plugins}; ${skills}`];
  }
  if (e.type === "system" && e.subtype === "permission_denied") {
    return [`✕ denied ${e.tool_name}: ${short(e.message, 100)}`];
  }
  if (e.type === "assistant") {
    return (e.message?.content ?? []).flatMap((c) => {
      if (c.type === "tool_use") return [`→ ${what(c.name, c.input)}`];
      if (c.type === "text" && c.text?.trim()) return full ? [`says:\n${c.text.trim()}`] : [`says: ${short(c.text)}`];
      return [];
    });
  }
  if (e.type === "user") {
    return (e.message?.content ?? []).flatMap((c) => {
      if (c.type !== "tool_result" || !c.is_error) return [];
      const text = Array.isArray(c.content) ? c.content.map((x) => x.text ?? "").join(" ") : c.content;
      return [`  ! ${short(text, 100)}`];
    });
  }
  if (e.type === "result") {
    const s = Math.round((e.duration_ms ?? 0) / 1000);
    const denied = (e.permission_denials ?? []).length;
    return [`■ ${e.is_error ? "ended with an error" : "done"}: ${Math.floor(s / 60)}m${String(s % 60).padStart(2, "0")}s, ` +
      `${e.num_turns ?? "?"} turns, ~$${(e.total_cost_usd ?? 0).toFixed(2)} at API prices` +
      (denied ? `, ${denied} denied` : "")];
  }
  return [];
}

// Each run's stream, how far it has been read, and any half line.
const seen = new Map();
function runs() {
  if (!fs.existsSync(dir)) return [];
  return fs.readdirSync(dir).filter((c) => c.startsWith("eval-")).flatMap((c) =>
    ["with_skill", "without_skill"].flatMap((config) => {
      const d = path.join(dir, c, config);
      return fs.existsSync(d) ? fs.readdirSync(d).filter((k) => k.startsWith("run-")).map((k) => path.join(d, k)) : [];
    })).filter((r) => !only || r.includes(only)).sort();
}
function poll() {
  for (const run of runs()) {
    const file = path.join(run, "stream.jsonl");
    if (!fs.existsSync(file)) continue;
    const s = seen.get(file) ?? { at: 0, rest: "" };
    const size = fs.statSync(file).size;
    if (size <= s.at) continue;
    const fd = fs.openSync(file, "r");
    const buf = Buffer.alloc(size - s.at);
    fs.readSync(fd, buf, 0, buf.length, s.at);
    fs.closeSync(fd);
    const lines = (s.rest + buf.toString("utf8")).split("\n");
    s.rest = lines.pop();
    s.at = size;
    seen.set(file, s);
    for (const line of lines) {
      let e;
      try { e = JSON.parse(line); } catch { continue; }
      for (const text of describe(e)) console.log(`[${label(run)}] ${text}`);
    }
  }
}

poll();
if (!once) setInterval(poll, 1000);
