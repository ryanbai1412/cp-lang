# A local library

What a contestant's own library looks like: a folder with a `lib.toml`,
one module per file, tests beside the code, and solutions that import
from it. It holds things the standard library leaves out, written on top
of it.

```text
examples/local-lib/
  lib.toml                    name = "lib"
  ds/rollback_dsu.cp          union-find with undo
  ds/li_chao.cp               lowest line at a point
  graph/two_sat.cp            2-SAT on std.graph's scc
  math/matrix.cp              matrices over Mod<P>
  solutions/road_reversal.cp  a problem solved with the library
```

```text
$ python3 proto/cp.py test examples/local-lib
$ printf '3 2\n1 2\n2 3\n' | python3 proto/cp.py lib-run examples/local-lib/solutions/road_reversal.cp
YES
[0, 1, 0]
```

## Importing

The name in `lib.toml` is the root of every import. A file anywhere below
the folder finds it by walking up, so a solution needs no configuration:

```
use lib.graph.two_sat.{TwoSat, Lit}

let (m, k): (Int, Int) = read()
domain Road = 1..=m
var sat = TwoSat<Road>()
for _ in 0..k:
    let (a, b): (Int, Int) = read()
    sat.differ(Lit(Road(a), true), Lit(Road(b), true))
if let plan = sat.solve():
    print("YES")
    print([if plan[r]: 1 else: 0 for r in Road])
else:
    print("NO")
```

A library module imports the standard library the same way a solution
does. `two_sat.cp` is short because `std.graph.scc` does the work:

```
use std.graph.{scc}

pub fn solve(self) -> ?[X]Bool:
    domain N = 0..2 * X.K
    let (C, comp) = scc::<N>(u => [(to: N(v)) for v in out[u.to_idx]])
    for x in X:
        if comp[N(2 * x.to_idx)] == comp[N(2 * x.to_idx + 1)]: return none
    [X] x => comp[N(2 * x.to_idx + 1)] > comp[N(2 * x.to_idx)]
```

## The conventions it follows

Only `pub` items are visible to an importer; helpers stay private to
their file. A module holds declarations only, so importing it cannot run
code.

Structures are indexed by a domain. `RollbackDsu<V>` takes vertices,
`TwoSat<X>` takes variables, and `solve` returns a `[X]Bool` that can
only be read with a variable.

Absence is `none`, not a sentinel. `LiChao.min_at` returns `?Int`
because a tree with no lines has no minimum; `TwoSat.solve` returns
`none` when the clauses cannot be satisfied.

A type with arithmetic implements the operator traits and then works
everywhere a number does. `Mat<P>` implements `Mul`, so `pow` is the
usual square-and-multiply, and `Mat<P>.product(n)` packages it as a
`Monoid` for a segment tree of matrices.

A structure with no invariant between its cells may lend them.
`Mat<P>.at(i, j)` is a projection (`lends mut Mod<P>`), so
`m.at(i, j) += x` writes into the matrix and nothing is copied. The
structures that keep an invariant (a union-find, a Li Chao tree) expose
operations instead, so no caller can break them.

Every test checks against something simpler: `rollback_dsu.cp` replays
thirty random unions and checks each `undo` restores the earlier state;
`li_chao.cp` compares with a scan over all lines and then solves a DP
both ways; `two_sat.cp` tries all 32 assignments.

## Starting your own

Copy the folder, keep `lib.toml`, and delete what you do not use. To use
a library that lives elsewhere, name it in the project's `cp.toml`:

```toml
[libs]
lib = "../my-cp-lib"
```
