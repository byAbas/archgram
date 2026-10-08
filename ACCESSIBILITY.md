# Accessibility

archgram draws diagrams for people to read, so a diagram must be readable
by everyone it is drawn for, including people who use a screen reader,
need high contrast or turn motion off. Being accessible is one of
archgram's principles ([docs/PRD.md](docs/PRD.md), 4. Principles).

## What archgram commits to

- **Contrast:** WCAG 2.1 AA, in the light theme and in the dark one. A
  project's own colours are held to the same rules, and archgram refuses
  them when they miss
  ([DESIGN.md](DESIGN.md), Colors).
- **Motion:** under reduced motion the diagram stands still, and the still
  image carries the whole meaning
  ([DESIGN.md](DESIGN.md), Layout).
- **A screen reader:** every diagram has a title and a description, which
  a spec must give, and a screen reader hears each flow in words
  ([docs/SPEC.md](docs/SPEC.md), Top level and Flows).

## Known limitations

- A diagram is one image to a screen reader. Its title and description,
  and its flows, are read in words; the text on the cards and the lines is
  not read one part at a time ([DESIGN.md](DESIGN.md), Credit). The
  description has to say what the drawing shows.
- How well a description reads depends on whoever writes the spec.
  archgram requires one but cannot judge it.
- The optional "by archgram" credit, a logotype, keeps no text contrast
  (WCAG 1.4.3 exempts logotypes); a screen reader skips it, and
  `credit: false` leaves it out ([DESIGN.md](DESIGN.md), Credit).

Where a diagram is shown, and how it follows dark mode there, is in the
[README](README.md) (FAQ).

## Reporting a barrier

If something in archgram, a drawing or the documentation keeps you from
using it, open an
[accessibility barrier issue](https://github.com/byAbas/archgram/issues/new?template=accessibility.yml).
It is worth reporting even when it is listed above. If you prefer not to
report it in public, email the maintainer at
[me@abasturabli.com](mailto:me@abasturabli.com).

## Contributing

A change that alters the drawing keeps to the rules above; how to try
one in both themes is in [CONTRIBUTING.md](CONTRIBUTING.md). The
project's maintainer looks after this statement and updates it when what
archgram does changes.
