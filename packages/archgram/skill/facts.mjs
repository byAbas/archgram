#!/usr/bin/env node
// Holds the archgram skill's drawings to the facts the user approved
// (SKILL.md, step 2 and step 6; references/facts.md, Held to approval).
// It ships in the archgram npm package as `archgram-facts`, outside the
// skill's folder, which holds text alone. Run it from the project's root:
//
//   npx --package archgram@X.Y.Z archgram-facts approve [--not-asked]
//   npx --package archgram@X.Y.Z archgram-facts verify
//
// `approve` stamps each section of docs/diagrams/facts.md with the hash of
// its text, once the user has approved it (`--not-asked` when the request
// said to go on without asking). `verify`, before each build, checks every
// spec in docs/diagrams/ against the stamped document: the approved text
// is unchanged; every approved relation is drawn, or not drawn with its
// reason; every edge is an approved relation under its approved label; a
// diagram split from an approved one answers all of its questions; one
// diagram draws one phase unless its layer says the user asked for more.
// It reads only those files, writes only the stamps, and uses no network.
//
// Exit codes: 0 approved or verified, 1 a problem found (stop and ask the
// user), 2 a wrong use or a missing document.

import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';

const FOLDER = 'docs/diagrams';
const DOCUMENT = path.join(FOLDER, 'facts.md');
const STAMP = /^<!-- approved: sha256:([0-9a-f]{64})( \(not asked\))? -->$/;
// More questions than this make a drawing of more than one idea
// (references/reader.md, The reader contract).
const QUESTIONS = 3;

function fail(message) {
  console.error(`facts: ${message}`);
  process.exit(2);
}

// The document as sections, each `## ` heading with its lines; what comes
// before the first heading is kept as it is.
function sections(text) {
  const lines = text.replace(/\r\n/g, '\n').split('\n');
  const head = [];
  const out = [];
  for (const line of lines) {
    if (line.startsWith('## ')) out.push({ heading: line.slice(3).trim(), lines: [] });
    else if (out.length) out[out.length - 1].lines.push(line);
    else head.push(line);
  }
  for (const s of out) {
    s.stamp = null;
    s.notAsked = false;
    const first = s.lines.findIndex((l) => l.trim() !== '');
    const m = first >= 0 ? s.lines[first].trim().match(STAMP) : null;
    if (m) {
      s.stamp = m[1];
      s.notAsked = Boolean(m[2]);
      s.lines.splice(first, 1);
    }
    s.body = s.lines.join('\n').replace(/^(?:[ \t]*\n)+/, '').replace(/\s+$/, '');
  }
  return { head, sections: out };
}

const hash = (body) => createHash('sha256').update(body.split('\n').map((l) => l.replace(/\s+$/, '')).join('\n')).digest('hex');

// The rows of the first table whose header holds `columns`, as objects.
function table(body, columns) {
  const lines = body.split('\n');
  for (let i = 0; i < lines.length; i++) {
    const cells = row(lines[i]);
    if (!cells || !columns.every((c) => cells.includes(c))) continue;
    const rows = [];
    for (let j = i + 2; j < lines.length && row(lines[j]); j++) {
      const values = row(lines[j]);
      rows.push(Object.fromEntries(cells.map((c, k) => [c, (values[k] ?? '').trim()])));
    }
    return rows;
  }
  return null;
}

function row(line) {
  const t = line.trim();
  if (!t.startsWith('|') || !t.endsWith('|')) return null;
  return t.slice(1, -1).split('|').map((c) => c.trim());
}

// A part's name as a table writes it, without Markdown's marks.
const plain = (s) => s.replace(/^~~|~~$/g, '').replace(/`/g, '').trim();

// The line `Label: value` in a section, or null.
function field(body, label) {
  const m = body.match(new RegExp(`^\\s*(?:\\*\\*)?${label}(?:\\*\\*)?\\s*:\\s*(.+)$`, 'mi'));
  return m ? m[1].trim() : null;
}

// The reader contract's numbered questions: `1. …` lines in the section.
const questions = (body) => body.split('\n').filter((l) => /^\s*\d+\.\s+\S/.test(l)).length;

const numbers = (s) => (s ?? '').split(/[,\s]+/).filter(Boolean).map(Number).filter(Number.isInteger);

// --- The spec, as much of it as this needs -------------------------------

// A spec's nodes and edges: JSON as it is; YAML in the forms the skill
// writes (references/kinds/architecture/spec.md), a list item per node or
// edge, as a flow mapping on one line or a block of `key: value` lines.
function readSpec(file) {
  const text = readFileSync(file, 'utf8');
  if (file.endsWith('.json')) {
    const spec = JSON.parse(text);
    return {
      diagram: spec.diagram ?? 'architecture',
      title: spec.title ?? '',
      subtitle: spec.subtitle ?? '',
      nodes: spec.nodes ?? [],
      edges: spec.edges ?? [],
    };
  }
  const lists = { nodes: [], edges: [] };
  let diagram = 'architecture';
  const top = { title: '', subtitle: '' };
  let list = null;
  let item = null;
  // The indent of the list's own items: a deeper `- ` is a value's list,
  // such as an edge's sources, not a new item.
  let itemIndent = -1;
  for (const raw of text.replace(/\r\n/g, '\n').split('\n')) {
    if (!raw.trim() || raw.trim().startsWith('#')) continue;
    const indent = raw.length - raw.trimStart().length;
    const line = raw.trim();
    if (indent === 0) {
      item = null;
      itemIndent = -1;
      const [key, value] = split(line);
      list = key in lists ? lists[key] : null;
      if (key === 'diagram') diagram = scalar(value);
      if (key === 'title' || key === 'subtitle') top[key] = scalar(value);
      continue;
    }
    if (!list) continue;
    if (line.startsWith('- ') && (itemIndent < 0 || indent === itemIndent)) {
      item = {};
      itemIndent = indent;
      list.push(item);
      const rest = line.slice(2).trim();
      if (rest.startsWith('{')) Object.assign(item, flow(rest));
      else if (rest) put(item, rest);
    } else if (item && indent > itemIndent) {
      put(item, line);
    }
  }
  return { diagram, ...top, nodes: lists.nodes, edges: lists.edges };
}

function put(item, line) {
  const [key, value] = split(line);
  if (key && value !== undefined && value !== '') item[key] = scalar(value);
}

function split(line) {
  const i = line.indexOf(':');
  return i < 0 ? [line, undefined] : [line.slice(0, i).trim(), line.slice(i + 1).trim()];
}

// `{ a: 1, b: "x, y" }`: commas inside quotes or brackets do not split.
function flow(text) {
  const inner = text.trim().replace(/^\{/, '').replace(/\}$/, '');
  const out = {};
  let depth = 0;
  let quote = null;
  let start = 0;
  const parts = [];
  for (let i = 0; i < inner.length; i++) {
    const c = inner[i];
    if (quote) {
      if (c === quote && !(quote === '"' && inner[i - 1] === '\\')) quote = null;
    } else if (c === '"' || c === "'") quote = c;
    else if (c === '[' || c === '{') depth++;
    else if (c === ']' || c === '}') depth--;
    else if (c === ',' && depth === 0) {
      parts.push(inner.slice(start, i));
      start = i + 1;
    }
  }
  parts.push(inner.slice(start));
  for (const p of parts) {
    const [key, value] = split(p.trim());
    if (key && value !== undefined && !value.startsWith('[')) out[key] = scalar(value);
  }
  return out;
}

function scalar(value) {
  const v = (value ?? '').trim();
  if (v.startsWith('"') && v.endsWith('"')) {
    try {
      return JSON.parse(v);
    } catch {
      return v.slice(1, -1);
    }
  }
  if (v.startsWith("'") && v.endsWith("'")) return v.slice(1, -1).replace(/''/g, "'");
  return v;
}

// --- The commands -----------------------------------------------------------

function approve(notAsked) {
  if (!existsSync(DOCUMENT)) fail(`${DOCUMENT} is not there: write it first (SKILL.md, step 2)`);
  const { head, sections: all } = sections(readFileSync(DOCUMENT, 'utf8'));
  const out = [...head];
  for (const s of all) {
    out.push(`## ${s.heading}`, '', `<!-- approved: sha256:${hash(s.body)}${notAsked ? ' (not asked)' : ''} -->`);
    if (s.body) out.push('', s.body);
    out.push('');
  }
  writeFileSync(DOCUMENT, out.join('\n').replace(/\n+$/, '\n'));
  console.log(`facts: ${all.length} sections approved${notAsked ? ', the request said not to ask' : ''}`);
}

function verify() {
  if (!existsSync(DOCUMENT)) fail(`${DOCUMENT} is not there: write it first (SKILL.md, step 2)`);
  const { sections: all } = sections(readFileSync(DOCUMENT, 'utf8'));
  const problems = [];
  const notes = [];
  const say = (m) => problems.push(m);

  // 1. Approved text unchanged; every section approved, or split from one.
  const byHeading = new Map(all.map((s) => [s.heading, s]));
  for (const s of all) {
    const from = field(s.body, 'Split from');
    if (s.stamp) {
      if (hash(s.body) !== s.stamp) say(`"${s.heading}" changed after it was approved: put it back, or ask the user to approve the change`);
    } else if (!from) {
      say(`"${s.heading}" is not approved: ask the user to approve it`);
    } else if (!byHeading.get(from)?.stamp) {
      say(`"${s.heading}" is split from "${from}", which is not an approved section`);
    }
  }

  // The core's parts, with their phases.
  const core = all.find((s) => table(s.body, ['Part', 'Phase']) || table(s.body, ['Part', 'File']));
  const parts = core ? (table(core.body, ['Part', 'File']) ?? []) : [];
  const phaseOf = new Map(parts.map((p) => [plain(p.Part), p.Phase ?? '']));
  const hasPhase = parts.some((p) => 'Phase' in p);

  const layers = all.filter((s) => /^[A-Z][a-z]+: \S/.test(s.heading));
  for (const layer of layers.filter((l) => l.stamp)) {
    const [kind, name] = layer.heading.split(/: (.+)/);
    const splits = layers.filter((l) => !l.stamp && field(l.body, 'Split from') === layer.heading);
    const drawnBy = [layer, ...splits].map((l) => ({ layer: l, spec: specFile(l.heading.split(/: (.+)/)[1]) }))
      .filter((d) => d.spec);
    const n = questions(layer.body);
    if (n > QUESTIONS) notes.push(`"${layer.heading}" asks ${n} questions; one idea is at most ${QUESTIONS} (references/reader.md)`);

    // 4. A split answers every question of the layer it was split from.
    if (!specFile(name) && splits.length) {
      const answered = new Set(splits.flatMap((l) => numbers(field(l.body, 'Answers'))));
      for (let q = 1; q <= n; q++) {
        if (!answered.has(q)) say(`question ${q} of "${layer.heading}" is answered by none of the diagrams split from it`);
      }
    }
    if (kind !== 'Architecture') continue;

    const relations = (table(layer.body, ['From', 'To', 'What']) ?? []).filter((r) => !r.From.startsWith('~~'));
    const notDrawn = [layer, ...splits].flatMap((l) => table(l.body, ['From', 'To', 'Why']) ?? []);
    const drawn = drawnBy.flatMap(({ spec }) => edgesOf(spec, parts));

    // 2. Every approved relation drawn, or not drawn with its reason.
    for (const r of relations) {
      const key = `${plain(r.From)} → ${plain(r.To)}`;
      if (drawn.some((e) => e.from.has(plain(r.From)) && e.to.has(plain(r.To)))) continue;
      const left = notDrawn.find((x) => plain(x.From) === plain(r.From) && plain(x.To) === plain(r.To));
      if (!left) say(`${key} (${layer.heading}) is approved but drawn nowhere, and not listed under Not drawn with its reason`);
      else if (!left.Why) say(`${key} is not drawn, and its row under Not drawn gives no reason`);
    }

    // 3. Every edge an approved relation, under its approved label.
    for (const e of drawn) {
      const matches = relations.filter((r) => e.from.has(plain(r.From)) && e.to.has(plain(r.To)));
      if (!matches.length) {
        say(`${e.file}: the edge ${e.label0} is no approved relation of "${layer.heading}"`);
      } else if (e.label && !matches.some((r) => r.What === e.label)) {
        say(`${e.file}: the edge ${e.label0} is labelled "${e.label}", approved as "${matches.map((r) => r.What).join('" or "')}": a new label needs the user's approval`);
      }
    }

    // 5. One diagram, one phase, unless the user asked for more.
    if (hasPhase) {
      for (const { layer: l, spec } of drawnBy) {
        const phases = new Set(nodesOf(spec, parts).flatMap((nd) => nd.parts.map((p) => phaseOf.get(p)).filter(Boolean)));
        const asked = field(l.body, 'Phases');
        if (phases.size > 1 && !asked) {
          say(`${spec}: draws ${[...phases].join(' and ')}; one diagram draws one phase, unless the user asked for more and the layer says so in a line "Phases: …"`);
        }
      }
    }
  }

  // 6. Diagrams that share a title tell themselves apart by a subtitle.
  const byTitle = new Map();
  for (const l of layers) {
    const file = specFile(l.heading.split(/: (.+)/)[1]);
    if (!file) continue;
    const { title, subtitle } = readSpec(file);
    const key = String(title).trim();
    byTitle.set(key, [...(byTitle.get(key) ?? []), { file, subtitle: String(subtitle ?? '').trim() }]);
  }
  for (const [title, specs] of byTitle) {
    if (specs.length < 2) continue;
    const subs = specs.map((x) => x.subtitle);
    if (subs.some((x) => !x) || new Set(subs).size < subs.length) {
      say(`${specs.map((x) => x.file).join(' and ')} share the title "${title}": give each a subtitle of its own`);
    }
  }

  for (const n of notes) console.log(`facts: note: ${n}`);
  if (problems.length) {
    for (const p of problems) console.log(`facts: ${p}`);
    console.log(`facts: ${problems.length} problem${problems.length === 1 ? '' : 's'}; stop and ask the user`);
    process.exit(1);
  }
  console.log(`facts: verified, ${all.filter((s) => s.stamp).length} approved sections, ${layers.length} diagrams`);
}

function specFile(name) {
  if (!name) return null;
  for (const ext of ['.archgram.yaml', '.archgram.yml', '.archgram.json']) {
    const file = path.join(FOLDER, name + ext);
    if (existsSync(file)) return file;
  }
  return null;
}

// Each node with the parts it stands for: its label, and any part its
// note names (a merged node, references/kinds/architecture/spec.md).
function nodesOf(file, parts) {
  const spec = readSpec(file);
  return spec.nodes.map((nd) => {
    const label = plain(String(nd.label ?? ''));
    const note = String(nd.note ?? '');
    const names = new Set([label]);
    for (const p of parts) if (p.Part && note.includes(plain(p.Part))) names.add(plain(p.Part));
    return { id: nd.id, parts: [...names] };
  });
}

function edgesOf(file, parts) {
  const spec = readSpec(file);
  if (spec.diagram !== 'architecture') return [];
  const nodes = new Map(nodesOf(file, parts).map((nd) => [nd.id, new Set(nd.parts)]));
  return spec.edges.map((e) => ({
    file,
    from: nodes.get(e.from) ?? new Set(),
    to: nodes.get(e.to) ?? new Set(),
    label: e.label ? String(e.label).trim() : null,
    label0: `${e.from} → ${e.to}`,
  }));
}

const [command, flag] = process.argv.slice(2);
if (command === 'approve' && (!flag || flag === '--not-asked')) approve(flag === '--not-asked');
else if (command === 'verify' && !flag) verify();
else fail('use: archgram-facts approve [--not-asked] | archgram-facts verify');
