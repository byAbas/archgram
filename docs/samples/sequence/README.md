# Samples: a sequence diagram's two looks

Hand-made samples that chose the two looks in
[docs/features/sequence.md](../../features/sequence.md) (Looks), kept so
they can be compared later. They are not archgram's output: they were
drawn by hand, in archgram's colours, icons and logos, before the engine
drew them.

The story is the same in both: a browser posts a login to an API, which
verifies the session with an external auth service; if the password
matches, the API finds the user, compares the hash and returns a session,
and otherwise refuses with a 401.

| File | Look |
|---|---|
| `cards.{light,dark}.svg` | `cards`, the default: card heads, activation bars, a solid frame with its pill at the left |
| `avatars.{light,dark}.svg` | `avatars`: round heads, no activation bars, a dashed frame with its pill in the middle |

The first samples chosen from tinted every other band and named the
guards in plain words ("if password matches", "otherwise"); these follow
DESIGN.md instead: every band filled alike, so a connector keeps 3:1
against it, and UML's operator in the pill, `alt · password matches`.
