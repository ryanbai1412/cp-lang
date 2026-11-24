# Friction report: coci21c6p5

Rethought. The v0.2 port pushed 0-based pairs into an untyped `List<(Int, Int)>` and indexed everything by `Int` positions in `0..2 * n`. It skipped with `continue` twice. The v0.3 port reads the pairs at once as `[Pair](a: Pos, b: Pos)` over `domain Pos = 1..=2 * n`, so positions stay 1-based and checked, and names each character with `bracket(p)`. The heap holds `(Pos, Pair)`, and the scan nests its two conditions instead of using `continue`.

No language gap.
