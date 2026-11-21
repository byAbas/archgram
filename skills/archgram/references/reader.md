# The reader

A diagram is judged in the first thirty seconds by someone who did not
build the system. Decide who that is and what they need before writing a
single node; everything else serves this.

## Contents

- Who reads it
- The reader contract
- Questions by kind of system
- Picking the shape
- The critique

## Who reads it

Take the reader from what the user said, or the document they named for
the diagram; else from the project itself, who uses it and who works on
it. Never from a README the user did not name. Ask only when nothing tells
you.

| Reader | What they are doing | What they expect |
|---|---|---|
| New contributor | Finding where to make a change | Names that exist in the repository; what calls what; where data lives; what is generated and must not be edited |
| Domain expert | Judging whether the architecture is sound | The mechanism: where a model is called, where checks happen, what runs at build time and what at run time |
| Reviewer of a task | Checking each requirement was met | Each requirement visible, and the one decision that makes the solution good shown rather than claimed |
| Stakeholder | Understanding what the system does | Few nodes, plain words, one flow from input to outcome |
| Operator | Running or deploying it | Services, stores, external systems, what runs where |

A page usually has two readers. The diagram serves the one who will judge
it; the prose serves the other.

## The reader contract

Write it in a few lines, at the top of the diagram's section of the facts
document (`references/facts.md`), its questions numbered, at most three:

```
Reader: <who>
They must be able to answer, from the diagram alone:
  1. <question>
  2. <question>
  3. <question>
The one idea the picture must make obvious: <the non-obvious property>
Left out on purpose: <what, and where it is explained instead>
```

The one idea is what tells this system apart from the obvious version of
it: "the app checks the model's citations", "code never reads the
palette", "every write goes through the ledger". If the diagram would look
the same for any system of its kind, it has not found the idea. A flow in
the spec is how the idea moves: make the contract's main path a flow.

One idea, and at most three questions: a fourth is usually a second
diagram. A diagram that answers three questions well beats one that
answers ten badly. When the user asks for more in one diagram, draw what
they asked, and say in the report what it costs the reader.

## Questions by kind of system

Start from these and keep the one to three the reader cares most about,
all about one phase: what runs, or how it is built, or what is set up
before use. A question about another phase is another diagram's.

- **Web app.** What happens between the user's action and the response?
  What runs in the browser, on the server, in a third-party service? Where
  is state kept, and who writes it? Where are the trust boundaries?
- **LLM, RAG or agent systems.** What does the model see, and never see?
  Which calls are model calls, and which deterministic code? Exact or
  semantic retrieval? What stops the model inventing facts, and where?
  Indexing ahead and answering at question time are two phases: two
  diagrams, the store one writes and the other reads drawn in both.
- **Services and events.** Which services exist and which own which data?
  What is a call, what a message on a queue? What happens when one is down?
- **Data pipelines.** Where does data enter, change and land? Batch or
  streaming, on what schedule? Where is it validated, where does bad data
  go?
- **Build pipelines, design systems, code generation.** What is written by
  hand, what generated, what must never be edited? What does the build
  read and write, in what order? What enforces the rules: lint, hooks, CI?
- **CI/CD.** What triggers a build, what does it run, where does it
  deploy? Which environments, and where are the secrets?

## Picking the shape

| The reader's main question | In the spec |
|---|---|
| "What happens, in order?" | One flow along the main path, `direction: right` |
| "What is authored, built, generated, consumed?" | Frames for the stages, in flow order |
| "What runs before use, and what at run time?" | One diagram for each phase, the store between them drawn in both |
| "Which part owns what?" | A frame for each owner, edges only for the calls that matter |
| "What checks what?" | A frame for the checks, each edge naming what it reads |

Two readers with different questions may need two diagrams: say so rather
than cramming both into one, and write two specs.

## The critique

Run it on the previews `build --preview` writes, the drawing as a reader
sees it (SKILL.md, step 6), before you report, and fix what it finds.
Judge what the image shows, not what the spec says it should.

1. **Contract.** Answer each question from the preview alone, and write
   the answer down. A question the image cannot answer is a gap: fix the
   spec, not the answer.
2. **The one idea.** Is it visible without reading the report? Its parts
   stand where the eye lands, near the middle or on the main path, not
   pushed to an edge, and the lines that carry it are not lost among
   others.
3. **Truth.** Is every node backed by a file you read, and named after
   the file that decides (`references/kinds/architecture/find.md`)? Any
   name that does not exist in the code?
4. **Edges.** Does every edge have the line of code that makes it, and
   mean the same kind of thing (a call, data moving)? Open each line you
   cite and check it does what the edge says: a line number read from
   memory drifts. "Contains", "configures" and "references" are frames,
   or said in the report, not edges.
5. **Level of detail.** Is an implementation detail shown while a
   load-bearing idea is missing? Is anything there only because it exists
   in the code? More than about 10 nodes and 12 edges, or two nodes with
   the same relations that no question tells apart
   (`references/kinds/architecture/spec.md`, Same relations, one node)?
6. **Missing steps.** Follow each flow from its first step to its last:
   where would the reader ask "how did it get here?", such as a browser
   that comes back from a payment page it was never sent to? Any hop the
   data takes that the picture skips: an embedding call, a build step, a
   check? Any entry point, mode, output or enforcement at this level that
   is neither drawn nor listed under Not drawn with its reason
   (`references/kinds/architecture/find.md`, What is easy to miss)?
7. **Width and counts.** No wider than keeps its text readable where it
   is shown: 1,300 px where no place is named, `shownWidth × 1300 / 880`
   where one is (SKILL.md, step 6); over it, `build` warns with what
   brings it within. `build`'s counts at 0: lines that cross, lines
   through a frame, lines running back, cards reached side by side.
8. **Clutter.** An edge label longer than two or three words; a label on
   an edge that could be part of a node's name; a line that detours round the
   whole drawing; arrowheads crowding into one card, or lines crossing
   where another order would not; a frame that groups nothing a reader
   needs.
9. **Both themes.** Every name, kind, label and logo readable on the light and
   on the dark preview; a logo or a colour that disappears on one of
   them is named in the report.
10. **Honest gaps.** What the diagram still does not show. Say it in the
   report; do not hide it.
