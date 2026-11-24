# Friction report: egoi23p8

Rethought. The v0.2 port packed each composition into a base-12 `Int` from a global recursive `gen` writing into `codes` and a `rank` map, took digits back out with a division loop, and used `rank.get(code) ?? codes.len` as a sentinel. It counted with hand loops and tracked the unseen slot with `slot`/`k` counters. The v0.3 port lists the compositions as digit lists in lexicographic order from a pure `compositions(parts, total)`. It walks one with `.items`, finds the unseen house among the 12 with `find`, splits run 2 labels into `ones`/`coded`/`flags` with `filter`, and ranks Emma's composition by comparing whole lists. The package gains gen.py and a run-twice interactor: Emma's house, the order of the others and her label (1, K or random) come from the seed.

No language gap.

## Measurements

N = 1e5: both runs together 0.26 s under the interactor. Port, v0.2 port and accepted C++ (91/100, K = 16) pass all 71 generated cases.
