# What an architecture diagram draws

## Contents

- What goes in
- How to read
- Names and labels
- The spec

## What goes in

The C4 container level (https://c4model.com/diagrams/container): the
people and clients that start the system, the project's own apps and
services, the stores they use, and the outside systems their code calls.
A part goes in only when the project's own code addresses it: it is that
code, or the code names it through a client, a library, a URL or a
config key it reads. What no code addresses (what only routes traffic
to the system, hosts it or reaches into it from outside) belongs to a
deployment view: leave it out, and say so in the report.

The modules, routes and files inside one app are not boxes: draw them
only when the user asks for the inside of one app. One diagram draws
one phase, what runs; how it is built or set up is another diagram.

## How to read

Read only what this takes, in this order, and stop when every box and
every line has its file and line:

1. `git ls-files`, for the layout: names only.
2. Each app's manifest (the file its package manager reads): each one an
   app, its dependencies the outside systems it may call.
3. The run files (compose, Dockerfile, a deploy manifest): what runs on
   its own.
4. Each app's entry point: how it starts, what it registers, which
   clients it makes.
5. For each relation, search for the call and open that line alone: it
   is the line's source.

What you did not read is not drawn.

## Names and labels

- Every box built on a technology has its logo. Ask archgram once, with
  every technology the manifests and run files name (packages, images,
  products): `archgram logos <name>...` prints the `tech` each one takes,
  or its candidates to choose from, or that it has none.
- Name a box for what it is or does here. With a logo, never the
  technology's name alone, which the logo shows.
- A line's label is the act of its source line: one verb and one
  object, three words at most (`stores users`, `starts checkout`). No
  second act, no list, no article, no single word, no object as vague
  as `data`.
- The people and clients that start the system go in `hints.first`.

## The spec

- A node per box with its kind, `tech` when it has a logo, and its file
  as `source`; an edge per relation with its source line; a person or an
  outside system has no file.
- A frame only for a boundary the reader needs (the system against the
  outside), never around one app's insides or a deployment.
- One flow, the main path, so it animates.
- Inside `{ }`, quote every label: a comma ends a value.
- Leave `direction: auto` to archgram; once the drawing passes, write
  the direction `build` chose.
