// The archgram skill's facts script, packages/archgram/skill/facts.mjs
// (`archgram-facts` in the npm package),
// with Node's own test runner: `node --test scripts/skill-facts.test.mjs`.
// Each test writes a facts document and specs into a temporary project,
// approves the document, and reads what `verify` says.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const script = path.join(path.dirname(fileURLToPath(import.meta.url)), '../packages/archgram/skill/facts.mjs');

function project(files) {
  const dir = mkdtempSync(path.join(tmpdir(), 'archgram-facts-'));
  for (const [name, text] of Object.entries(files)) {
    const full = path.join(dir, 'docs/diagrams', name);
    mkdirSync(path.dirname(full), { recursive: true });
    writeFileSync(full, text);
  }
  return dir;
}

function facts(dir, ...args) {
  const run = spawnSync(process.execPath, [script, ...args], { cwd: dir, encoding: 'utf8' });
  return { code: run.status, out: run.stdout + run.stderr };
}

const CORE = `# Facts

## The project

| Part | What it is | Technology | File | Phase | Evidence |
|---|---|---|---|---|---|
| Customer | a browser | | none | run | 1 |
| API | HTTP API | express | ../../src/api.ts | run | 1 |
| orders | job queue | redis | ../../src/queue.ts | run | 1 |
| Worker | job worker | | ../../src/worker.ts | run | 1 |
| Postgres | database | postgresql | ../../src/db.ts | run | 1 |
| Bundler | build step | vite | ../../vite.config.ts | build | 1 |
`;

const LAYER = `## Architecture: architecture

Reader: a new contributor. They must be able to answer, from the diagram alone:
  1. What happens to an order?
  2. Where is it kept?

| From | To | What | Source | Evidence |
|---|---|---|---|---|
| Customer | API | POST /orders | ../../src/api.ts#app.post( | 1 |
| API | orders | enqueue | ../../src/api.ts#orders.add( | 1 |
| orders | Worker | take a job | ../../src/worker.ts#new Worker( | 1 |
| Worker | Postgres | insert order | ../../src/worker.ts#saveOrder( | 1 |
`;

const NODES = `nodes:
  - { id: customer, kind: browser, label: Customer }
  - { id: api, kind: service, label: API }
  - { id: orders, kind: queue, label: orders }
  - { id: worker, kind: service, label: Worker }
  - { id: db, kind: database, label: Postgres }
`;

const ALL_EDGES = `edges:
  - { from: customer, to: api, label: POST /orders }
  - { from: api, to: orders, label: enqueue }
  - { from: orders, to: worker, label: "take a job" }
  - from: worker
    to: db
    label: insert order
    source:
      - "../../src/worker.ts#saveOrder("
`;

const spec = (body) => `archgram: 1\ntitle: shop\ndescription: d\n${body}`;

test('a document nobody approved stops the drawing', () => {
  const dir = project({ 'facts.md': CORE + '\n' + LAYER, 'architecture.archgram.yaml': spec(NODES + ALL_EDGES) });
  const run = facts(dir, 'verify');
  assert.equal(run.code, 1);
  assert.match(run.out, /"The project" is not approved/);
});

test('approved and drawn as approved, it passes; the stamps stay put on a second approval', () => {
  const dir = project({ 'facts.md': CORE + '\n' + LAYER, 'architecture.archgram.yaml': spec(NODES + ALL_EDGES) });
  assert.equal(facts(dir, 'approve').code, 0);
  const once = readFileSync(path.join(dir, 'docs/diagrams/facts.md'), 'utf8');
  assert.match(once, /<!-- approved: sha256:[0-9a-f]{64} -->/);
  const run = facts(dir, 'verify');
  assert.equal(run.code, 0, run.out);
  assert.match(run.out, /verified/);
  facts(dir, 'approve');
  assert.equal(readFileSync(path.join(dir, 'docs/diagrams/facts.md'), 'utf8'), once, 'the same text, the same stamps');
});

test('approved text edited afterwards is caught', () => {
  const dir = project({ 'facts.md': CORE + '\n' + LAYER, 'architecture.archgram.yaml': spec(NODES + ALL_EDGES) });
  facts(dir, 'approve');
  const file = path.join(dir, 'docs/diagrams/facts.md');
  writeFileSync(file, readFileSync(file, 'utf8').replace('| insert order |', '| save order |'));
  const run = facts(dir, 'verify');
  assert.equal(run.code, 1);
  assert.match(run.out, /"Architecture: architecture" changed after it was approved/);
});

test('a label the user did not approve is caught; a quoted comma is one value', () => {
  const edges = ALL_EDGES.replace('label: enqueue', 'label: "enqueue, retry"');
  const dir = project({ 'facts.md': CORE + '\n' + LAYER, 'architecture.archgram.yaml': spec(NODES + edges) });
  facts(dir, 'approve');
  const run = facts(dir, 'verify');
  assert.equal(run.code, 1);
  assert.match(run.out, /labelled "enqueue, retry", approved as "enqueue"/);
});

test('an edge with no approved relation behind it is caught', () => {
  const edges = ALL_EDGES + '  - { from: customer, to: db }\n';
  const dir = project({ 'facts.md': CORE + '\n' + LAYER, 'architecture.archgram.yaml': spec(NODES + edges) });
  facts(dir, 'approve');
  const run = facts(dir, 'verify');
  assert.equal(run.code, 1);
  assert.match(run.out, /customer → db is no approved relation/);
});

test('an approved relation left out needs a reason under Not drawn', () => {
  const edges = ALL_EDGES.replace('  - { from: customer, to: api, label: POST /orders }\n', '');
  const dir = project({ 'facts.md': CORE + '\n' + LAYER, 'architecture.archgram.yaml': spec(NODES + edges) });
  facts(dir, 'approve');
  let run = facts(dir, 'verify');
  assert.equal(run.code, 1);
  assert.match(run.out, /Customer → API \(Architecture: architecture\) is approved but drawn nowhere/);

  const why = LAYER + '\n| From | To | Why |\n|---|---|---|\n| Customer | API | the browser is the README\'s own picture |\n';
  const dir2 = project({ 'facts.md': CORE + '\n' + why, 'architecture.archgram.yaml': spec(NODES + edges) });
  facts(dir2, 'approve');
  run = facts(dir2, 'verify');
  assert.equal(run.code, 0, run.out);
});

test('a split needs no approval of its own, but must answer every question', () => {
  const titled = (sub, body) => spec(body).replace('title: shop', `title: shop\nsubtitle: ${sub}`);
  const first = titled('Taking an order', NODES + 'edges:\n  - { from: customer, to: api, label: POST /orders }\n  - { from: api, to: orders, label: enqueue }\n');
  const second = titled('Keeping an order', NODES + 'edges:\n  - { from: orders, to: worker, label: take a job }\n  - { from: worker, to: db, label: insert order }\n');
  const split = (name, answers) => `## Architecture: ${name}\n\nSplit from: Architecture: architecture\nAnswers: ${answers}\n`;
  const dir = project({ 'facts.md': CORE + '\n' + LAYER, 'first.archgram.yaml': first, 'second.archgram.yaml': second });
  facts(dir, 'approve');
  const file = path.join(dir, 'docs/diagrams/facts.md');
  const approved = readFileSync(file, 'utf8');
  writeFileSync(file, approved + '\n' + split('first', '1') + '\n' + split('second', '1'));
  let run = facts(dir, 'verify');
  assert.equal(run.code, 1);
  assert.match(run.out, /question 2 of "Architecture: architecture" is answered by none/);
  writeFileSync(file, approved + '\n' + split('first', '1') + '\n' + split('second', '2'));
  run = facts(dir, 'verify');
  assert.equal(run.code, 0, run.out);
});

test('one diagram draws one phase, unless its layer says the user asked for more', () => {
  const nodes = NODES + '  - { id: bundler, kind: script, label: Bundler }\n';
  const layer = LAYER + '| Bundler | API | static build | ../../vite.config.ts#outDir | 1 |\n';
  const edges = ALL_EDGES + '  - { from: bundler, to: api, label: static build }\n';
  const dir = project({ 'facts.md': CORE + '\n' + layer, 'architecture.archgram.yaml': spec(nodes + edges) });
  facts(dir, 'approve');
  let run = facts(dir, 'verify');
  assert.equal(run.code, 1);
  assert.match(run.out, /draws run and build; one diagram draws one phase/);
  const asked = layer.replace('Reader:', 'Phases: run, build (the user asked for the build in the same diagram)\n\nReader:');
  const dir2 = project({ 'facts.md': CORE + '\n' + asked, 'architecture.archgram.yaml': spec(nodes + edges) });
  facts(dir2, 'approve');
  run = facts(dir2, 'verify');
  assert.equal(run.code, 0, run.out);
});

test('more than three questions is a note, not a stop', () => {
  const four = LAYER.replace('  2. Where is it kept?', '  2. Where is it kept?\n  3. Who takes it?\n  4. What runs on its own?');
  const dir = project({ 'facts.md': CORE + '\n' + four, 'architecture.archgram.yaml': spec(NODES + ALL_EDGES) });
  facts(dir, 'approve');
  const run = facts(dir, 'verify');
  assert.equal(run.code, 0, run.out);
  assert.match(run.out, /note: "Architecture: architecture" asks 4 questions; one idea is at most 3/);
});

test('a request that said not to ask is stamped as such', () => {
  const dir = project({ 'facts.md': CORE + '\n' + LAYER, 'architecture.archgram.yaml': spec(NODES + ALL_EDGES) });
  assert.equal(facts(dir, 'approve', '--not-asked').code, 0);
  assert.match(readFileSync(path.join(dir, 'docs/diagrams/facts.md'), 'utf8'), /\(not asked\) -->/);
  assert.equal(facts(dir, 'verify').code, 0);
});

test('a wrong use, or no document, is refused', () => {
  const dir = project({});
  assert.equal(facts(dir, 'verify').code, 2);
  assert.equal(facts(dir, 'stamp').code, 2);
});

test('two diagrams with one title need a subtitle each', () => {
  const first = spec(NODES + 'edges:\n  - { from: customer, to: api, label: POST /orders }\n  - { from: api, to: orders, label: enqueue }\n');
  const second = spec(NODES + 'edges:\n  - { from: orders, to: worker, label: take a job }\n  - { from: worker, to: db, label: insert order }\n');
  const split = (name, answers) => `## Architecture: ${name}\n\nSplit from: Architecture: architecture\nAnswers: ${answers}\n`;
  const dir = project({ 'facts.md': CORE + '\n' + LAYER, 'first.archgram.yaml': first, 'second.archgram.yaml': second });
  facts(dir, 'approve');
  const file = path.join(dir, 'docs/diagrams/facts.md');
  writeFileSync(file, readFileSync(file, 'utf8') + '\n' + split('first', '1') + '\n' + split('second', '2'));
  let run = facts(dir, 'verify');
  assert.equal(run.code, 1);
  assert.match(run.out, /share the title "shop": give each a subtitle of its own/);
  writeFileSync(path.join(dir, 'docs/diagrams/first.archgram.yaml'), first.replace('title: shop', 'title: shop\nsubtitle: Taking an order'));
  writeFileSync(path.join(dir, 'docs/diagrams/second.archgram.yaml'), second.replace('title: shop', 'title: shop\nsubtitle: Keeping an order'));
  run = facts(dir, 'verify');
  assert.equal(run.code, 0, run.out);
});
