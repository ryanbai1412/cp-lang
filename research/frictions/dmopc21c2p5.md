# Friction report: dmopc21c2p5

No semantic gap. One table `ways[d][i]` per node, weighted by d! or (d + 1)! at the end, gives both of its answers (parent edge unused or used), where the old port ran the whole knapsack twice. `Deg = 0..=26` keeps the 26! = 0 cut-off in the type.

## Measurements

gen.py --max (n = 3000, k = 1500): port 1.11 s, accepted C++ 1.20 s.
