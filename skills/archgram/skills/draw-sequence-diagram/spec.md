# Writing a sequence spec


## Contents

- From what was found
- Phases
- How much to draw
- In the report
- An example

## From what was found

From what `find.md` found:
its Participants, its Messages in the order the code runs, its Branches
as fragments, and phases for its steps. A sequence has no `direction` and no
`flows`: it is drawn top to bottom, numbers its messages and moves on its
own, one phase at a time, so leave `still` out too. Inside `{ }`, quote
every `label` and `note` (`label: "find user, then check"`): there a comma
ends a value. A participant's `note` is never drawn.

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
that, leave out the calls the contract's questions do not need (a helper
inside one part, a log line), and say so in the report. Split the
conversation at a phase only when the user asks.

## In the report

Under "Not shown": what was left out past about 15 messages. Each
participant's file and each message's line stay in the spec's sources.

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
  - { id: auth, kind: auth, label: Session check, variant: external }
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
