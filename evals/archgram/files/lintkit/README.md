# lintkit

A small linter with plugins. `lintkit init` sets a project up, `check`
finds problems, `fix` repairs what it can and checks again, and `report`
writes a page for the team. Rules are plugins: each file in `plugins/` is
handed lintkit's API and adds its rules through it.
