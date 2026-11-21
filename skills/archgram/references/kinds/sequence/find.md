# Finding a sequence in the code

A sequence diagram answers a different question from architecture: not
what the system is made of, but in what order its parts call each other
for one request, and what comes back. Its meaning is UML's (clause 17,
Interactions). This says how to find that order in the code;
`references/kinds/sequence/spec.md` says how it becomes the spec, and
`spec --section sequence` what the spec may hold.

## Contents

- One conversation per diagram
- Participants
- Messages in the order the code runs
- Fragments from the code's branches
- A refusal
- From a document
- In the facts document

## One conversation per diagram

Pick one entry point and one scenario the reader cares about: "signing in
with a password", "placing an order", "a cache miss". Start where the
request comes in (a route, a command, a message handler) and follow it in
the order the code runs it to the response. Two scenarios are two
diagrams; what the system is made of is architecture's. A sequence skips
the styles of architecture.

## Participants

A participant is a part the request passes through, at one level for the
whole diagram, usually the containers of
`references/kinds/architecture/find.md` (an app, a service, a database, a
queue, an external system), with the caller first: the browser or person
who sends the request. List them left to right in the order the request
first reaches them, so most lines point right on the way in and left on
the way back. Give each its kind, its technology's logo (`tech`) and,
when a file backs it, that file as its `source`; a system you call but do
not own is `variant: external`. With a logo, name it for what it does here
(`Users` with `tech: postgresql`), not for the technology the logo
already shows.

## Messages in the order the code runs

Walk the handler line by line and write a message for each call the
reader needs, in that order, with the line that makes it as its `source`
(`src/routes/login.ts#auth.verifySession(`). That is the call in the
handler the reader follows, not a line inside the function it calls: a
`fetch` or a query inside `auth.ts` is how the call is done, and the
reader looks for the call where the request is handled.

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

## From a document

When the user names a document, each message's `source` is the sentence
that states it (`docs/login.md#the API asks Auth to verify the session`),
in the document's own words; what the document does not state is not
drawn, as for architecture.

## In the facts document

The parts go into the document's core (`references/facts.md`). The
diagram's layer, `## Sequence: <name>`, its name the scenario's
(`sign-in`), holds besides its reader and what was expected and not
found:

- **Participants**: `| Part | File |`, the core's parts the request
  passes through, in the order it first reaches them;
- **Messages**: `| # | From | To | Message | Type | Source | Evidence |`,
  in the order the code runs them, `Type` one of `call` (waits),
  `send` (does not), `reply`, `self` and `refused`;
- **Branches**: `| Fragment | Guard | Messages | Source |`, each `alt`,
  `opt`, `loop` or `par`, its guard in the reader's words, the messages
  it holds by their `#`, and the line of its `if`.
