#!/usr/bin/env node
// Measures what the archgram skill would read of a project, before it reads
// any of it (SKILL.md, Size the scope): the files git tracks, less what
// references/ignore.md leaves out, in areas no larger than one reader's
// budget. Run it from the project's root:
//
//   node <the skill's folder>/scripts/size.mjs [--budget TOKENS] [PATH ...]
//
// PATHs narrow it to the scope, folders or files under the project; with
// none it measures the whole project. It prints Markdown: what it would
// read, the areas, what it leaves out and why, and the manifests it saw.
// It reads only under the project's folder, follows no symbolic link,
// prints no file's contents and uses no network. The same files give the
// same output on every run.
//
// Exit codes: 0 measured, 2 a wrong use or a path outside the project.

import { execFileSync } from 'node:child_process';
import { closeSync, lstatSync, openSync, readdirSync, readFileSync, readSync } from 'node:fs';
import path from 'node:path';

// About as many tokens as one reader takes in well, a first guess to be
// measured (docs/features/facts.md, Open questions).
const BUDGET = 100000;
// Bytes per token, a rough rule for code: the counts are estimates.
const BYTES_PER_TOKEN = 4;
// Past this a file is not read, as archgram reads none larger (SECURITY.md).
const LARGEST = 4 * 1024 * 1024;
// Rows a table prints before it sums up the rest.
const ROWS = 40;
const HEAD = 1024;

// references/ignore.md, layer 3: folders and files never the project's own.
const NEVER_DIRS = new Set([
  'vendor', 'third_party', 'node_modules', 'bower_components', 'Pods',
  '.venv', 'venv', '__pycache__',
]);
const LOCK_FILES = new Set([
  'package-lock.json', 'pnpm-lock.yaml', 'yarn.lock', 'Cargo.lock', 'go.sum',
  'poetry.lock', 'npm-shrinkwrap.json', 'bun.lock', 'Gemfile.lock', 'composer.lock',
]);
const GENERATED_NAMES = [/^zz_generated/, /\.pb\.go$/, /_pb2(_grpc)?\.py$/];
const GO_GENERATED = /^\/\/ Code generated .* DO NOT EDIT\.$/m;
// Images, fonts and other media, read by no walk of the code.
const MEDIA = /\.(svg|png|jpe?g|gif|webp|avif|ico|bmp|tiff?|woff2?|ttf|otf|eot|pdf|mp3|mp4|webm|mov|wav|ogg|zip|gz|tgz|wasm)$/i;

const MANIFESTS = new Set([
  'package.json', 'Cargo.toml', 'go.mod', 'go.work', 'pyproject.toml', 'requirements.txt',
  'setup.py', 'pom.xml', 'build.gradle', 'build.gradle.kts', 'Gemfile', 'composer.json',
  'pnpm-workspace.yaml', 'Chart.yaml', 'serverless.yml', 'serverless.yaml',
  'compose.yml', 'compose.yaml', 'Dockerfile',
]);
const isManifest = (name) =>
  MANIFESTS.has(name) || /^Dockerfile\./.test(name) || /^docker-compose.*\.ya?ml$/.test(name) ||
  /\.csproj$/.test(name);

const WHY = {
  never: "never the project's own (references/ignore.md, 3)",
  lock: 'a lock file (references/ignore.md, 3)',
  minified: 'minified or a source map (references/ignore.md, 3)',
  generatedName: 'generated, by its name (references/ignore.md, 3)',
  generatedHead: 'generated, by its first lines (references/ignore.md, 3)',
  media: 'an image, a font or other media (references/ignore.md, 3)',
  marked: 'marked in .gitattributes (references/ignore.md, 2)',
  ignored: 'git ignores it (references/ignore.md, 1)',
  untracked: 'git does not track it (references/ignore.md, 1)',
  link: 'a symbolic link, not followed',
  binary: 'not text',
  large: `larger than ${LARGEST / (1024 * 1024)} MiB`,
};

function usage(message) {
  process.stderr.write(
    `size.mjs: ${message}\n\nUsage: node size.mjs [--budget TOKENS] [PATH ...]\n`,
  );
  process.exit(2);
}

function args(argv) {
  let budget = BUDGET;
  const paths = [];
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === '--budget') {
      budget = Number(argv[++i]);
      if (!Number.isInteger(budget) || budget < 1000) usage('--budget takes a whole number of tokens, 1000 or more');
    } else if (argv[i].startsWith('-')) {
      usage(`unknown option ${argv[i]}`);
    } else {
      paths.push(argv[i]);
    }
  }
  return { budget, paths };
}

// The scope's paths, relative to the project, each one under it and there.
function scope(root, paths) {
  return paths.map((p) => {
    const full = path.resolve(root, p);
    const rel = path.relative(root, full);
    if (rel.startsWith('..') || path.isAbsolute(rel)) usage(`${p} is outside the project`);
    try {
      lstatSync(full);
    } catch {
      usage(`${p} is not there`);
    }
    return rel.split(path.sep).join('/') || '.';
  });
}

function git(root, argv, input) {
  return execFileSync('git', argv, {
    cwd: root,
    input,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024 * 1024,
    stdio: ['pipe', 'pipe', 'ignore'],
  });
}

function inGit(root) {
  try {
    return git(root, ['rev-parse', '--is-inside-work-tree']).trim() === 'true';
  } catch {
    return false;
  }
}

const nul = (text) => text.split('\0').filter(Boolean);

// Without git: every file under the scope. A folder never the project's
// own is not walked into, only noted, since it can hold more files than
// the rest of the project.
function walk(root, start, found, skipped) {
  const full = path.join(root, start);
  let entries;
  try {
    entries = readdirSync(full, { withFileTypes: true });
  } catch {
    found.push(start);
    return;
  }
  for (const e of entries.sort((a, b) => (a.name < b.name ? -1 : 1))) {
    if (e.name === '.git') continue;
    const rel = start === '.' ? e.name : `${start}/${e.name}`;
    if (e.isDirectory() && NEVER_DIRS.has(e.name)) skipped.push(`${rel}/`);
    else if (e.isDirectory()) walk(root, rel, found, skipped);
    else found.push(rel);
  }
}

// Linguist's marks for each path: true when set, false when unset with a
// leading `-`, which keeps a path the built-in list would leave out.
function marks(root, files) {
  const marked = new Map();
  if (files.length === 0) return marked;
  const out = nul(git(root, ['check-attr', '-z', '--stdin', 'linguist-vendored', 'linguist-generated'], files.join('\0')));
  for (let i = 0; i + 2 < out.length; i += 3) {
    const [file, value] = [out[i], out[i + 2]];
    if (value === 'set' || value === 'true') marked.set(file, true);
    else if ((value === 'unset' || value === 'false') && !marked.has(file)) marked.set(file, false);
  }
  return marked;
}

function head(full) {
  const fd = openSync(full, 'r');
  try {
    const buf = Buffer.alloc(HEAD);
    const n = readSync(fd, buf, 0, HEAD, 0);
    return buf.subarray(0, n);
  } finally {
    closeSync(fd);
  }
}

// Why a file is left out, or null when it is read.
function why(root, file, mark) {
  const parts = file.split('/');
  const name = parts.at(-1);
  if (mark === true) return WHY.marked;
  const kept = mark === false;
  if (!kept && parts.slice(0, -1).some((p) => NEVER_DIRS.has(p))) return WHY.never;
  if (!kept && LOCK_FILES.has(name)) return WHY.lock;
  if (!kept && (/\.min\.js$/.test(name) || /\.map$/.test(name))) return WHY.minified;
  if (!kept && GENERATED_NAMES.some((re) => re.test(name))) return WHY.generatedName;
  if (!kept && MEDIA.test(name)) return WHY.media;
  const full = path.join(root, file);
  const stat = lstatSync(full);
  if (stat.isSymbolicLink()) return WHY.link;
  if (!stat.isFile()) return WHY.binary;
  if (stat.size > LARGEST) return WHY.large;
  const first = head(full);
  if (first.includes(0)) return WHY.binary;
  if (!kept) {
    const text = first.toString('utf8');
    const lines = text.split('\n').slice(0, 5).join('\n');
    if (GO_GENERATED.test(text) || lines.includes('@generated')) return WHY.generatedHead;
  }
  return null;
}

function measure(root, file) {
  const buf = readFileSync(path.join(root, file));
  let lines = 0;
  for (let at = buf.indexOf(10); at !== -1; at = buf.indexOf(10, at + 1)) lines++;
  if (buf.length > 0 && buf[buf.length - 1] !== 10) lines++;
  return { file, bytes: buf.length, lines };
}

const tokens = (bytes) => Math.ceil(bytes / BYTES_PER_TOKEN);

// The read files as areas, each a folder no larger than the budget: a
// folder over it is split into its folders, and its own files as one
// more area, as far as its folders go.
function areas(read, budget) {
  const tree = { path: '', dirs: new Map(), files: [] };
  for (const f of read) {
    let node = tree;
    const parts = f.file.split('/');
    for (const p of parts.slice(0, -1)) {
      if (!node.dirs.has(p)) node.dirs.set(p, { path: node.path ? `${node.path}/${p}` : p, dirs: new Map(), files: [] });
      node = node.dirs.get(p);
    }
    node.files.push(f);
  }
  const sum = (node) => {
    if (node.total) return node.total;
    const t = { files: node.files.length, lines: 0, bytes: 0 };
    for (const f of node.files) {
      t.lines += f.lines;
      t.bytes += f.bytes;
    }
    for (const d of node.dirs.values()) {
      const s = sum(d);
      t.files += s.files;
      t.lines += s.lines;
      t.bytes += s.bytes;
    }
    node.total = t;
    return t;
  };
  const label = (node) => (node.path ? `${node.path}/` : './');
  const own = (node) => {
    const t = { files: node.files.length, lines: 0, bytes: 0 };
    for (const f of node.files) {
      t.lines += f.lines;
      t.bytes += f.bytes;
    }
    return t;
  };
  const out = [];
  const split = (node) => {
    const t = sum(node);
    if (t.files === 0) return;
    if (tokens(t.bytes) <= budget || node.dirs.size === 0) {
      out.push({ area: label(node), ...t });
      return;
    }
    for (const name of [...node.dirs.keys()].sort()) split(node.dirs.get(name));
    if (node.files.length) out.push({ area: `${label(node)} (its own files)`, ...own(node) });
  };
  split(tree);
  return out.sort((a, b) => (a.area < b.area ? -1 : a.area > b.area ? 1 : 0));
}

// Left-out files in rows: by why, and by the folder that says it, the one
// never the project's own, or else the file's own folder.
function leftOut(files) {
  const rows = new Map();
  for (const { file, why: reason } of files) {
    const parts = file.split('/');
    let at = parts.slice(0, -1).findIndex((p) => NEVER_DIRS.has(p));
    const where = reason === WHY.never && at >= 0
      ? `${parts.slice(0, at + 1).join('/')}/`
      : parts.length > 1 ? `${parts.slice(0, -1).join('/')}/` : './';
    const key = `${where}\0${reason}`;
    rows.set(key, (rows.get(key) ?? 0) + 1);
  }
  return [...rows.entries()]
    .map(([key, n]) => {
      const [where, reason] = key.split('\0');
      return { where, n, reason };
    })
    .sort((a, b) => (a.where < b.where ? -1 : a.where > b.where ? 1 : a.reason < b.reason ? -1 : 1));
}

function table(header, rows, cells) {
  const lines = [`| ${header.join(' | ')} |`, `|${header.map(() => '---').join('|')}|`];
  for (const row of rows.slice(0, ROWS)) lines.push(`| ${cells(row).join(' | ')} |`);
  if (rows.length > ROWS) lines.push(`| and ${rows.length - ROWS} more |${' |'.repeat(header.length - 1)}`);
  return lines.join('\n');
}

function main() {
  const root = process.cwd();
  const { budget, paths } = args(process.argv.slice(2));
  const within = scope(root, paths);
  const git_ = inGit(root);

  let tracked;
  let ignored = [];
  let untracked = [];
  let skipped = [];
  if (git_) {
    const spec = within.length ? ['--', ...within] : [];
    tracked = nul(git(root, ['ls-files', '-z', ...spec]));
    ignored = nul(git(root, ['ls-files', '-z', '--others', '--ignored', '--exclude-standard', '--directory', ...spec]));
    untracked = nul(git(root, ['ls-files', '-z', '--others', '--exclude-standard', '--directory', ...spec]));
  } else {
    tracked = [];
    for (const start of within.length ? within : ['.']) walk(root, start, tracked, skipped);
  }
  tracked = [...new Set(tracked)].sort();

  const marked = git_ ? marks(root, tracked) : new Map();
  const read = [];
  const out = [];
  for (const file of tracked) {
    let reason;
    try {
      reason = why(root, file, marked.get(file));
    } catch {
      continue; // listed by git, but gone from the working tree
    }
    if (reason) out.push({ file, why: reason });
    else read.push(measure(root, file));
  }

  const total = { files: read.length, lines: 0, bytes: 0 };
  for (const f of read) {
    total.lines += f.lines;
    total.bytes += f.bytes;
  }
  const parts = areas(read, budget);
  const budgets = Math.max(1, Math.ceil(tokens(total.bytes) / budget));
  const size = tokens(total.bytes) <= budget ? 'small' : budgets <= 8 ? 'medium' : 'large';

  const lines = [];
  lines.push('# What the archgram skill would read');
  lines.push('');
  lines.push(`Scope: ${within.length ? within.join(', ') : 'the whole project'}`);
  lines.push(`Read: ${total.files} files, ${total.lines} lines, about ${tokens(total.bytes)} tokens (bytes / ${BYTES_PER_TOKEN}, an estimate)`);
  lines.push(`One reader's budget: about ${budget} tokens`);
  lines.push(`Size: ${size}, ${parts.length} ${parts.length === 1 ? 'area' : 'areas'}`);
  if (!git_) lines.push('Not a git repository: what .gitignore and .gitattributes say was not applied');
  lines.push('');
  lines.push('## Areas');
  lines.push('');
  lines.push(table(['Area', 'Files', 'Lines', 'About tokens'], parts, (a) => [a.area, a.files, a.lines, tokens(a.bytes)]));
  lines.push('');
  lines.push('## Left out');
  lines.push('');
  const left = [
    ...ignored.map((p) => ({ where: p, n: '', reason: WHY.ignored })),
    ...untracked.map((p) => ({ where: p, n: '', reason: WHY.untracked })),
    ...skipped.map((p) => ({ where: p, n: '', reason: WHY.never })),
    ...leftOut(out),
  ].sort((a, b) => (a.where < b.where ? -1 : a.where > b.where ? 1 : a.reason < b.reason ? -1 : 1));
  lines.push(left.length ? table(['Path', 'Files', 'Why'], left, (r) => [r.where, r.n, r.reason]) : 'Nothing.');
  lines.push('');
  lines.push('## Manifests');
  lines.push('');
  const manifests = read.map((f) => f.file).filter((f) => isManifest(f.split('/').at(-1)));
  lines.push(manifests.length ? manifests.slice(0, ROWS).map((m) => `- ${m}`).join('\n') + (manifests.length > ROWS ? `\n- and ${manifests.length - ROWS} more` : '') : 'None.');
  process.stdout.write(`${lines.join('\n')}\n`);
}

main();
