# Writing a sequence spec

`references/kinds/sequence/find.md` found the participants, each with its
file, and the messages, in the order the code runs them, each with its
line. This says how they become the spec; the fields are in
`spec --section sequence` (SKILL.md, step 3).

## Contents

- The facts in the format
- Phases
- How much to draw
- In the report
- An example

## The facts in the format

From the diagram's layer of the facts document (`references/facts.md`):
its Participants, its Messages in the order the code runs, its Branches
as fragments, and phases for its steps. A sequence has no `direction` and no
`flows`: it is drawn top to bottom, numbers its messages and moves on its
own, one phase at a time, so leave `still` out too. Inside `{ }`, quote
every `label` and `note` (`label: "find user, then check"`): there a comma
ends a value, and the words after it read as a field.

## Phases

Divide the conversation into its steps, two to four, each a few words in
the reader's terms: `phase: Sign in`, `phase: Check the password`. A
phase is an item of the top-level `messages` list and holds the messages
after it, up to the next; it never sits inside a fragment, and every
phase has a message after it. The drawing plays one phase at a time and
names it, so the phases are how the reader follows the story: pick them
from what the reader asks, not from the code's functions.

## How much to draw

About 15 messages is as much as a reader follows on one screen. Past
that, split the conversation in two, at a phase, and draw each as its own
diagram; or leave out calls the contract's questions do not need (a
helper inside one part, a log line). Say what you split or left out in
the report.

## In the report

Under "Not shown" (SKILL.md, step 7): what was split or left out past
about 15 messages. Each participant's file and each message's line stay
in the facts document.

## An example

```yaml
archgram: 1
diagram: sequence
title: Sign in
description: >-
  The browser posts the login to the API, which verifies the session with
  the auth service and checks the password against the users table; with
  the right password it returns a session, and otherwise refuses with a
  401.
participants:
  - { id: browser, kind: browser, label: Browser }
  - { id: api, kind: service, label: API, tech: express, source: ../../src/routes/login.ts }
  - { id: auth, kind: service, label: Auth, variant: external }
  - { id: db, kind: database, label: Users, tech: postgresql, source: ../../src/db.ts }
messages:
  - phase: Sign in
  - { from: browser, to: api, label: POST /login, source: "../../src/routes/login.ts#router.post(\"/login\"" }
  - phase: Check the password
  - { from: api, to: db, label: find user, source: "../../src/routes/login.ts#findUser(" }
  - { reply: db, label: user row }
  - alt:
      - when: password matches
        source: "../../src/routes/login.ts#if (await matches("
        messages:
          - { reply: api, label: 200 · session }
      - when: else
        messages:
          - { reply: api, label: "401", refused: true }
```
