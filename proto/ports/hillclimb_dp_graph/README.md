# Hillclimb: hard DP, graph and tree problems

Fifteen accepted submissions from `corpus/`, each solved again from scratch
in CP Lang rather than translated. Where a port needed boilerplate, the
language or the library was changed and the port rewritten; the earlier
version is kept in `before/` so the drop can be measured. The decisions are
in `decisions.md` section 12.

DP: `dpo` (bitmask matching), `dpu` (subset partition), `dps` (digit DP),
`dpp` (tree DP), `dpv` (rerooting), `noip06p1` (interval DP on a ring),
`apio07p3` (window bitmask on a ring).
Graphs and trees: `apio09p3` (SCC condensation), `coi06p2` (bridges and cut
vertices), `noi15p2` (heavy-light paths with a lazy tree), `sac22cc3p4`
(offline connectivity with LCA), `ioi11p4` (second-best Dijkstra).
Constructive: `ccc22s3`, `dmopc21c5p1`, `yac6p2`.

Each `x.cp` is checked against `x_orig.cpp` on random inputs by
`proto/run_all.sh`. Line counts, raw and library-stripped:

```
python3 proto/loc.py hillclimb_dp_graph
```

All fifteen: 279 lines against 1446 (81% fewer), and against 735 once the
originals lose what a perfect C++ library would have supplied (62% fewer).
The twelve ports with a `before/` version went from 364 lines to 235;
their library-stripped reduction went from 43% to 63%.

## What changed, by example

Subsets are sets (`dpu`). Before, the submask walk left the type:

```
var g = s.idx
while g > 0:
    best[s] max= score[Group.nth(g)] + best[Group.nth(s.idx ^ g)]
    g = (g - 1) & s.idx
```

After:

```
for g in s.subsets:
    best[s] max= score[g] + best[s - g]
```

Bridges without a hand-written DFS (`coi06p2`). `Rooted(map, root)` is the
depth-first spanning tree, so every other road joins a town to an ancestor:

```
let tree = Rooted(map, Town.first)
var low: [Town]Int = [Town] v => tree.pos(v)
for v in tree.at.rev:
    for w in map[v]:
        if tree.parent(v) != w: low[v] min= tree.pos(w)
    if let p = tree.parent(v): low[p] min= low[v]
```

A DP over strongly connected components (`apio09p3`):

```
let city = Graph.directed(roads)
let (District, of) = scc(city)
let onward = condense(city, of)
for d in District.rev: ...          // roads lead to later districts
```

Paths and subtrees as ranges (`noi15p2`, 76 lines before, 28 after):

```
var installed = AssignSum<tree.Pre>()
for (l, r) in tree.chains(Pkg.first, pkg):
    fresh += r.idx - l.idx + 1 - installed.sum(l, r)
    installed.assign(l, r, 1)
```

Also: `v.cyc(k)` for rings, `xs + ys` for lists, `Dsu<V>()`,
`tree.children(v)`, `Graph.undirected(edges)`, and read ranges such as `let m: n - 1..=5e5 = read()` that are
checked and generated with the bounds as written.

## Friction left

- No `Float` in the prototype compiler; `noi07p1` was swapped for
  `sac22cc3p4` because of it.
- `let (a, b): (T, U) = read()` and `var (lo, hi) = ...` are not accepted.
- Every tree port writes the same two-line random-tree generator.
- A tree given as a parent array needs a loop to build child lists
  (`yac6p2`, `noi15p2`).
- Rerooting (`dpv`) is still a hand-written prefix and suffix pass.
- Folding `?Int` values with `max` in a DAG sweep takes three lines
  (`apio09p3`).
