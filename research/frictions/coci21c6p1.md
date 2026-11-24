# Friction report: coci21c6p1

Rethought. The v0.2 port pushed names and totals into two untyped `List`s and counted the beekeepers ahead with a hand loop over `j` and a two-part `>`/`==` condition. The v0.3 port indexes both by `domain Keeper` and types n and the scores. It counts the ones ahead with `Keeper.count(..)` over one tuple comparison, `(total, desc(name))`, so "higher total, then smaller name" is a single ordering.

No language gap.
