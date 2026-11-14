# Samples: the marks of a diff

Hand-made samples that chose how `archgram diff` marks what a change
added, removed and changed
([docs/features/diff.md](../../features/diff.md), Decided 2), kept so the
other options can be compared later. They are not archgram's output:
each is `linkshort-change.archgram.json`, the old spec and the new one
written as one, drawn by archgram 0.10.1, with the marks and the legend
added by a script on top.

The story is the same in every sample, a change to
`examples/linkshort.json`: the worker that drained the click stream into
Postgres is removed, with its two lines; ClickHouse is added, reading the
click stream, and the API reads stats from it; the URL cache's TTL and
the label of the API's read from Postgres changed.

| File | The marks |
|---|---|
| `signs.{light,dark}.svg` | The look chosen. A sign in a filled circle, `+`, `−` or `~`, on a card's top edge and on a line; the card's border, the line and its arrowhead in the mark's colour; a removed card's title struck through |
| `mono.{light,dark}.svg` | The same signs with no colour: an added part's circle filled and its card's border in the text colour, a removed and a changed part's circle hollow, the lines as they are |
| `words.{light,dark}.svg` | A word on a card's top edge, `ADDED`, `REMOVED`, `CHANGED`, and a removed card hatched in place of the struck title; lines as in `signs`, since a word does not fit a short one |

The colours are the palette's green, red and amber. What the samples
showed besides the look is in the feature's document (Decided 2, Open
questions).
