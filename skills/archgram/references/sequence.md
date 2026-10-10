# Finding a sequence in the code

A sequence diagram answers a different question from architecture: not
what the system is made of, but in what order its parts call each other
for one request, and what comes back. Its meaning is UML's (clause 17,
Interactions). This says how to find that order in the code and write it
as a spec; `spec --section sequence` says what the spec may hold.

## Contents

- One conversation per diagram
- Participants
- Messages in the order the code runs
- Fragments from the code's branches
- A refusal
- Phases
- How much to draw
- From a document
- An example

## One conversation per diagram

Pick one entry point and one scenario the reader cares about: "signing in
with a password", "placing an order", "a cache miss". Start where the
request comes in (a route, a command, a message handler) and follow it in
the order the code runs it to the response. Two scenarios are two
diagrams; what the system is made of is architecture's.

## Participants

A participant is a part the request passes through, at one level for the
whole diagram, usually the containers of `references/architecture.md`
(an app, a service, a database, a queue, an external system), with the
caller first: the browser or person who sends the request. List them
left to right in the order the request first reaches them, so most
lines point right on the way in and left on the way back. Give each its
kind, its technology's logo (`tech`) and, when a file backs it, that file
as its `source`; a system you call but do not own is `variant: external`.

## Messages in the order the code runs

Walk the handler line by line and write a message for each call the
reader needs, in that order, with the line that makes it as its `source`
(`src/routes/login.ts#auth.verifySession(`):

- A call that waits for its result (an `await`, a synchronous call, a
  query) is a message: `{ from: api, to: db, label: find user }`.
- What comes back is a reply, written where the result returns, after
  whatever the callee did meanwhile: `{ reply: db, label: user row }`.
  It goes back to the latest call to `db` still waiting; leave `to` out
  unless the check asks for it. Its `source` is optional: the `return`
  line, when one says what comes back.
- A call nothing waits for (a queue's `add`, an event's `emit`, a
  promise not awaited) is a send: `async: true`, and no reply.
- A step inside one part is a message to itself (`from: api, to: api`),
  drawn only when the reader needs it ("compare hash"). It waits for a
  reply as any call does; if `archgram check` says a later reply answers
  it, mark it `async: true` when nothing comes back the reader needs.

A label is a few words, two or three, naming the call or the data:
`POST /login`, `find user`, `200 · session`. Long detail belongs in the
report.

## Fragments from the code's branches

A branch the reader needs becomes a fragment holding its own messages:

| The code | The fragment |
|---|---|
| `if … else …`, a `switch`, a `try` and its `catch`, each way calling something different | `alt`, each way an operand with its guard in `when`, the last `else` |
| `if` with no `else` | `opt`, its guard in `when` |
| `for`, `while`, `.map` over items | `loop`, its `when` saying over what: `for each page` |
| `Promise.all`, work started together | `par`, each operand one strand |

A guard is the condition in the reader's words (`password matches`, not
`bcrypt.compare(p, h)`), and its operand's `source` is the line of the
`if`. Leave out a branch that only logs or retries, unless the contract's
questions are about it.

## A refusal

The response that turns the request away, a 401, a 403, a validation
error, is a reply with `refused: true`, usually in an `alt`'s `else`:

```yaml
- alt:
    - when: password matches
      messages:
        - { reply: api, label: 200 · session }
    - when: else
      messages:
        - { reply: api, label: "401", refused: true }
```

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

## From a document

When the user names a document, each message's `source` is the sentence
that states it (`docs/login.md#the API asks Auth to verify the session`),
in the document's own words; what the document does not state is not
drawn, as for architecture.

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
