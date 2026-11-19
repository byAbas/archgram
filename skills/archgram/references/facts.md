# The facts document

The facts come before the spec, in one document the user reads and
approves before anything is drawn (SKILL.md, step 2). It holds what the
project is made of once, and what each diagram drawn from it needs, so a
wrong fact is caught before it reaches a drawing, and the next diagram,
of any kind, starts from what is already found.

## Contents

- Where it lives
- The core layer
- A diagram's layer
- Evidence
- Sources
- On an update
- An example

## Where it lives

`docs/diagrams/facts.md`, beside the specs: one for the project. Create
it when it is missing. When it is there, start from it: keep what still
holds, and change what the code no longer says (On an update, below).

Write it as Markdown tables, one row a fact, so the user reads it at a
glance and the spec is written from it row by row. Each table has the
columns below and no others, but for the `Changed` column an update adds
while it waits for the user's approval (On an update).

## The core layer

What every diagram of the project shares, at the top of the document,
under `## The project`:

- **Parts**: `| Part | What it is | Technology | File | Evidence |`. A
  part is named as the code names it (the file that decides,
  `references/kinds/architecture/find.md`); what it is is a few words,
  such as `HTTP API` or `orders queue`; the technology is its Simple
  Icons slug when it has one; the file is the one behind it, or `none`
  for a person or a system outside the code.
- **Boundaries**: `| Boundary | Parts | Evidence |`: a service, a trust
  zone, the code that is not the project's own.
- **Entry points**: `| Entry point | File | Line |`: every way the system
  is set going, read or not (`references/kinds/architecture/find.md`,
  Start from the entry points).
- **Coverage**: `| Folder | Read | Why |`: every top-level folder, and any
  folder below it left out, with `yes` or `no`, and why: the reason
  `references/ignore.md` gives, or that the diagram did not need it.

## A diagram's layer

Each diagram drawn from the document has its own section after the core,
`## <Kind>: <name>`, its name the spec's (`## Architecture: architecture`,
`## Sequence: sign-in`), holding:

- **Reader**: the reader contract (step 1), as `references/reader.md`
  writes it;
- the tables the kind's `find.md` gives in its section In the facts
  document, such as an architecture's relations or a sequence's
  messages;
- **Expected, not found**: `| What | Why it was expected | Where it was looked for |`:
  what the style or the kind usually has and the code does not show,
  such as a store a microservice usually owns. It is never drawn;
- **Open questions**: `| Question | What the code says | What a document says |`:
  where two readings are equally likely, or the code and a document
  disagree. The approval asks each of them.

## Evidence

Every fact carries what backs it, as a level in its `Evidence` column:

| Level | What backs the fact | Reaches the spec |
|---|---|---|
| 1 | A line of code that makes it: a call, a read, a write, a message sent | Yes |
| 2 | A configuration or a manifest that wires it: a compose file, a chart, a dependency injection container, a route table | Yes, with its source |
| 3 | A name, a folder's convention, a generated file | No: an open question |
| 4 | Only a README, a document or a comment says so | No: an open question |

Drawing from a document the user named (SKILL.md, From a document), a
sentence that states the fact is its evidence, level 1 for that drawing;
nothing else is.

A generated file is not read (`references/ignore.md`), but a generated
client is a hint, level 3, that a call exists: find the call in the code
that is not generated, and give that line.

## Sources

A source is written as the spec will take it, from `docs/diagrams/`:
`../../src/api.ts#db.insert(order)`, the path and a few words copied from
one line (`spec --section sources`). The spec takes it as it is, so a
source the document gives is one `archgram check` can hold to the code.
Open each line before writing it: a line number read from memory drifts.

## On an update

A spec drawn before the project had this document names its sources:
start the document from them, the parts and lines the spec already has,
and go on as below.

Start from the document as it is. Run `archgram check` on each spec drawn
from it; for each source it says has lost its code, find where the code
went, if anywhere, and change or remove the row. Walk the code again for
what is new, since the check cannot see what the code gained. Give each
table you touch a last column, `Changed`, and mark each row you changed,
added or removed there in a few words, a removed row struck through
(`~~API~~`), so the user's approval reads what changed; once the user
approves, drop the struck rows and the column.

## An example

```markdown
## The project

| Part | What it is | Technology | File | Evidence |
|---|---|---|---|---|
| Customer | a browser | | none | 1 |
| API | HTTP API | express | ../../src/api.ts | 1 |
| orders | job queue | redis | ../../src/queue.ts | 1 |
| Worker | job worker | | ../../src/worker.ts | 1 |
| Postgres | database | postgresql | ../../src/db.ts | 1 |

| Folder | Read | Why |
|---|---|---|
| src/ | yes | |
| node_modules/ | no | always left out |
| scripts/ | no | the diagram did not need it |

## Architecture: architecture

Reader: a new contributor. They must be able to answer: what happens to
an order, where it is kept, what runs on its own.

| From | To | What | Source | Evidence |
|---|---|---|---|---|
| Customer | API | POST /orders | ../../src/api.ts#app.post("/orders" | 1 |
| API | orders | enqueue | ../../src/api.ts#orders.add( | 1 |
| orders | Worker | take a job | ../../src/worker.ts#new Worker( | 1 |
| Worker | Postgres | insert order | ../../src/worker.ts#saveOrder( | 1 |
```
