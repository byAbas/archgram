# Fonts

`source/Geist-Regular.ttf` and `source/Geist-Medium.ttf` are the static
TrueType files of Geist v1.7.2, and `source/GeistMono-Regular.ttf` and
`source/GeistMono-Medium.ttf` those of Geist Mono from the same release,
all from the release archive at
https://github.com/vercel/geist-font/releases/tag/v1.7.2
(`geist-font-v1.7.2.zip`, SHA-256
`7fc800d2ac6b92844895196e5041aca55d814c15db70c44f79b3b83ab82b04e2`), paths
`geist-font/Geist/ttf/` and `geist-font/GeistMono/ttf/`. They are licensed
under the SIL Open Font License 1.1, in `OFL.txt`, the archive's one
licence for both families; it declares no Reserved Font Name.

archgram measures text with these files and embeds a subset of them in each
SVG: Geist for every kind of diagram, Geist Mono for a sequence's words
between its heads. The subset keeps the font's copyright and licence entries in its `name`
table, as the licence asks.

The files archgram embeds, `Geist-Regular.ttf`, `Geist-Medium.ttf`,
`GeistMono-Regular.ttf` and `GeistMono-Medium.ttf` here, are written from
those by `cargo xtask fonts`: every character each maps, with only the
tables a renderer needs, so kerning, ligatures and the glyphs only they
reach, glyph names and hinting are left out. Each Geist file is about
49 KB instead of 126 KB, and each Geist Mono file about 62 KB instead of
149 KB. `tests/fonts.rs` checks that every character keeps its
advance, its outline and the line's metrics, and that the files are what
the task writes. They are a Modified Version under the licence, which they
carry in their `name` table.
