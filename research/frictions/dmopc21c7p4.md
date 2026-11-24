# Friction report: dmopc21c7p4

Rethought. The v0.2 port kept the events in two parallel lists, `q_kind` and `q_arg` (kind 0 or 1, with the argument meaning r or a rocket index). It sorted versions by a negated key `-(a - b)` and rebuilt the slot arrays with `Slot.first!` placeholders. Its segment-tree record had abbreviated fields, and the binary search was a hand-written `lo`/`hi` loop with a `pos_count + 1` "not found" value. The v0.3 port reads the events into an `enum Event` (`Ask(r)`, `Swap(t, to)`) and types every bound. It orders the versions with `Version.sort(by: v => desc(net))` and folds the two halves with `Slot.cuts.nth(k)`. It finds the shortest reaching prefix with `first_true`, and `fewest` returns `?Int`, so -1 appears only when printing.

No language gap.

## Measurements

gen.py --max (n = q = 2e5): port 0.28 s, v0.2 port 0.24 s, accepted C++ 0.30 s, same output. The generated C++ is about 36.8 KB, over the generic judge's 32 KiB source target (`cpl build` W0201), and the port has no shorter form today.
