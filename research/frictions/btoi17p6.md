# Friction report: btoi17p6

Rethought from dream.cp. The v0.2 port joined each child into its parent with a reverse label scan over `Kid`. The v0.3 port builds `Rooted.from_parents([V] v => Kid.at(v.ord).map(k => parent[k]))` and folds with `tree.up(Monoid<Pick>(..), lift:, finish:)`, as the dream does. The parents still read as `[Kid]V` over `domain Kid = 1..n`.

No language gap.

## Measurements

gen.py --max: port 0.05 s. The accepted C++ recurses and dies (signal 11) under an 8 MB stack; with an unlimited stack it takes 0.02 s and gives the same output.
