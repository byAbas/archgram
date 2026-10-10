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

## A lighter drawing, phases and motion

The first drawing was found hard to follow, so a second round of samples
chose a lighter one (docs/features/sequence.md, Decided 11 to 13), drawn
by hand in the same way:

| File | What it shows |
|---|---|
| `phases.{light,dark}.svg` | Phases as bands across the page, thin lifelines, labels as plain text, a call stronger than a reply |
| `motion.{light,dark}.svg` | The chosen motion: one phase at a time, each line drawn with its arrowhead lit as it arrives, a faint tint between the lifelines it joins, the `alt`'s ways as alternatives (`4a`, `4b`) |

The motion plays in a browser; a still preview shows its first moment.
