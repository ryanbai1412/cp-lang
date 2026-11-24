# Friction report: coci21c3p1

Rethought. The v0.2 port read untyped `Int`s, checked a block with an early-return loop, and returned the hit as an anonymous `(Int, Int)`, read back as `.0` and `.1`. The v0.3 port types n, k and the colours. It checks a block with one `all(..)` and returns `?(start:, len:)`, printing the pattern as the slice `a[b.start..b.start + b.len]`. The package gains gen.py, with planted repeats so both answers occur, and a checker.py, since any pattern is accepted.

No language gap.
