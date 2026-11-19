# Writing an architecture spec

`references/kinds/architecture/find.md` found the parts, each with its
file, and the edges, each with its line. This says how they become the
spec, how much of them to draw, and what an architecture keeps on an
update. The fields are in the format's short part, `spec --brief`
(SKILL.md, step 3).

## Contents

- The facts in the format
- Same relations, one node
- How much to draw
- Direction
- Still
- On an update
- In the report

## The facts in the format

Each part a node of the kind that fits it, with its technology's logo
(`tech`) when the part is built on one; each call or data movement an
edge; a boundary (a service, a trust zone, the plugin versus the project)
a frame; the path the contract's questions follow a flow, so it animates.
Keep labels to the words a reader needs: an edge's label is a few words,
two or three, naming the call or the data (`POST /orders`, `enqueue`,
`read-through`); what the reader needs beyond that goes in a node's
`note` or the report, not onto the line, since a long label pushes the
cards apart.

## Same relations, one node

Whatever the architecture, parts the reader need not tell apart can be
one node, and the drawing says so without knowing the style: two parts
are the same to the reader when the same nodes lead into them and they
lead to the same nodes. Tokens, a rules document and source code that
one command reads and another writes are one node, "design sources",
with their names in its note; a config file only a third command writes
stays its own. Merge only parts at the same level, and name the merged
node after what they are for, not after one of them. Each merged node's
note names the parts it stands for.

Keep a part apart, even with the same relations, when a question in the
reader contract is about it, or when it is the one idea.

## How much to draw

About 10 nodes and 12 edges is what a reader takes in within thirty
seconds; start there, and treat it as a limit to argue with rather than a
rule. Over it:

1. merge parts with the same relations (above);
2. move what the contract does not ask about to "left out on purpose";
3. split into two diagrams, each with its own contract: one per entry
   point, per phase, or per level.

Completeness means nothing the reader's questions need is missing, not
that everything in the code is drawn. A loop (a report read by the
command that changes what the report reads) makes the drawing tall;
keep it when the loop is the idea, split it when it is not.

## Direction

Leave the direction of a new drawing to archgram, `direction: auto`,
unless the user asked for one: it lays the drawing out left to right
while that keeps its text readable where it is shown, and otherwise in
the narrower direction.

Once the drawing passes the critique (SKILL.md, step 6), write the
direction `build` reported (`right` or `down`) in place of
`direction: auto`, and check again: the drawing stays the same bytes, and
a later edit cannot turn it on its side. Turn a drawing only when the
user asks or `build` warns that it no longer fits where it is shown, and
say so in the report.

## Still

Leave `still` out, so the drawing carries no extra text, unless the user
asks for it or the drawing is meant to be seen still (a PNG, print): then
`numbers` for one or two flows, whose step numbers a reader can follow on
the lines, and `legend` for more, whose flows read better as words. With
four flows, numbers pile up into badges such as `5,8,11,14` that a reader
cannot tie to a flow.

## On an update

Keep the spec's `direction` as it is written, and with none written,
write none: the spec is then drawn left to right, as `right` would draw
it. Only when it is `auto`, first draw the spec as it is (SKILL.md,
step 6) and write the direction `build` reports in its place, so the
update cannot turn the drawing on its side.

## In the report

Besides what SKILL.md, step 7, lists:

- the direction written in the spec;
- the style of architecture you recognised, by its name in
  `references/kinds/architecture/styles.md`, or that none fitted.
