# What is not read

Most of a repository is code the project did not write or does not edit
by hand. Reading it fills the context and draws parts that are not the
project's. Leave out, in this order, each layer catching what the one
before leaves:

1. **What commonly holds secrets**, tracked or not, and never opened,
   not even to look or count: `.env`, `.env.*`, `.npmrc`, `.pypirc`,
   `.netrc`, `.git-credentials`, `*.pem`, `*.key`, `*.p12`, `*.pfx`,
   `id_rsa`, `id_dsa`, `id_ecdsa`, `id_ed25519`, and anything in `.ssh`,
   `.aws`, `.gnupg` or `.docker`: the list archgram itself refuses as a
   source (`spec --section sources`). Its path may go into the coverage;
   nothing of it goes anywhere.
2. **What git does not track.** What the project's `.gitignore` files
   ignore: usually `node_modules/`, `dist/`, `out/`, `.next/`,
   `coverage/`.
3. **What the project marks.** Paths its `.gitattributes` gives
   `linguist-vendored` or `linguist-generated`, GitHub Linguist's marks
   for code that is not the project's own or is written by a tool.
4. **What is never the project's own**, tracked or not:
   - `vendor/`, `third_party/`, `node_modules/`, `bower_components/`,
     `Pods/`, `.venv/`, `venv/`, `__pycache__/`;
   - `*.min.js`, `*.map`, and lock files (`package-lock.json`,
     `pnpm-lock.yaml`, `yarn.lock`, `Cargo.lock`, `go.sum`,
     `poetry.lock`);
   - images, fonts and other media (`.svg`, `.png`, `.jpg`, `.woff2`,
     `.ttf`, `.pdf`, `.mp4`, archives), which no walk of the code reads;
     a theme's colours come from the styling code, not from them
     (`references/theme.md`);
   - a generated file, by its first lines (Go's
     `// Code generated … DO NOT EDIT.`, a `@generated` mark) or its name
     (`zz_generated*`, `*.pb.go`, `*_pb2.py`).

`build/`, `out/`, `dist/`, `target/` and `bin/` are left out only when git
ignores them or what they hold is generated: in some projects they hold
source, such as build scripts. Look before you leave one out.

The skill's sizing command, `archgram-size`, leaves out the same, from git's own lists
and `git check-attr`, and prints each with its layer (SKILL.md, step 2).
Everything left out goes into the facts document's coverage, with the
layer that left it out (`references/facts.md`, The core layer), so the
user sees what was not read. A generated file is not read, but a
generated client is a hint that a call exists (`references/facts.md`,
Evidence).

When the user names something to read or to leave out ("read `build/`",
"leave `legacy/` out"), do as they say, and write it in the coverage.
