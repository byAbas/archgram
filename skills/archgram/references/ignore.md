# What is not read

Most of a repository is code the project did not write or does not edit
by hand. Reading it fills the context and draws parts that are not the
project's. Leave out, in this order, each layer catching what the one
before leaves:

1. **What git does not track.** What the project's `.gitignore` files
   ignore: usually `node_modules/`, `dist/`, `out/`, `.next/`,
   `coverage/`.
2. **What the project marks.** Paths its `.gitattributes` gives
   `linguist-vendored` or `linguist-generated`, GitHub Linguist's marks
   for code that is not the project's own or is written by a tool.
3. **What is never the project's own**, tracked or not:
   - `vendor/`, `third_party/`, `node_modules/`, `bower_components/`,
     `Pods/`, `.venv/`, `venv/`, `__pycache__/`;
   - `*.min.js`, `*.map`, and lock files (`package-lock.json`,
     `pnpm-lock.yaml`, `yarn.lock`, `Cargo.lock`, `go.sum`,
     `poetry.lock`);
   - a generated file, by its first lines (Go's
     `// Code generated … DO NOT EDIT.`, a `@generated` mark) or its name
     (`zz_generated*`, `*.pb.go`, `*_pb2.py`).

`build/`, `out/`, `dist/`, `target/` and `bin/` are left out only when git
ignores them or what they hold is generated: in some projects they hold
source, such as build scripts. Look before you leave one out.

Everything left out goes into the facts document's coverage, with the
layer that left it out (`references/facts.md`, The core layer), so the
user sees what was not read. A generated file is not read, but a
generated client is a hint that a call exists (`references/facts.md`,
Evidence).

When the user names something to read or to leave out ("read `build/`",
"leave `legacy/` out"), do as they say, and write it in the coverage.
