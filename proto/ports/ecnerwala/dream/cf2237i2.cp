// Codeforces 2237 I2 (dream).
// Tree DP over a rooted tree whose edges carry '0', '1', or '?'.
// At each vertex, the DP tracks (depth, active, resolved):
//   - depth: max depth of a resolved/active chain in the processed children
//   - active in {Off, On, Deferred}: whether an open active path reaches this node
//     (`Deferred` tracks the inclusion-exclusion adjustment on '0'/'1'/'?' edges)
//   - resolved: Bool — whether a resolved chain already sits strictly deepest.
// Merging a child across edge e in {0, 1} enforces that at most one active and
// at most one resolved chain exist, a resolved chain is strictly deepest, and
// an active chain cannot follow a 0-edge or coexist with depth > 0.
use std.tree.Rooted

type Z = Mod<1_000_000_007>
enum Label: Zero | One | Any
enum Active: Off | On | Deferred

fn can_merge(d0: Int, ca: Active, cn: Bool, nd1: Int, n_na: Active, n_nn: Bool, na: Active, e: Int) -> Bool:
    if ca != Off and n_na != Off: return false
    if cn and n_nn: return false
    if ca != Off and e == 0: return false
    if (n_nn and d0 > nd1) or (cn and d0 <= nd1): return false
    if (n_na != Off and d0 > 0) or (ca == Deferred and na == Deferred): return false
    true

@!multitest
let n: 1..=500 = read()
domain V = 1..=n
let label_str: Str = read()
let label: [V]Label = [v => if v == V.first: Zero else: match label_str[v.ord - 1]:
    '0' => Zero
    '1' => One
    _ => Any]

var par: [V]?V = none
for u in V:
    let deg: Int = read()
    repeat deg: par[read<V>()] = u
let t = Rooted.from_parents(par)

var dp: [V]List<[Active, Bool]Z> = [v => []]
for u in t.Pre.rev:
    var cur: List<[Active, Bool]Z> = [[a, r => if r == false and (a == Off or a == Deferred): 1 else: 0]]
    for c in t.children(u):
        let child = dp[c]
        let nh = max(cur.len, child.len + 1)
        var next: List<[Active, Bool]Z> = [[a, r => 0] for _ in 0..nh]
        for (d0, row0) in cur.items:
            for ca in Active, cn in Bool where row0[ca, cn] != 0:
                for (d1, row1) in child.items:
                    for na in Active, nn in Bool where row1[na, nn] != 0:
                        let ways = row0[ca, cn] * row1[na, nn]
                        for e in 0..2:
                            if (label[c] == Zero and e == 1) or (label[c] == One and e == 0): continue
                            let (n_na, n_nn, nd1) = if e == 1 and na != Off: (Off, true, 1) else: (na, nn, d1 + e)
                            if not can_merge(d0, ca, cn, nd1, n_na, n_nn, na, e): continue
                            let nd = max(d0, nd1)
                            let hn = cn or n_nn
                            if ca == Deferred or na == Deferred:
                                match label[c]:
                                    Zero => if na == Deferred: next[nd][Deferred, hn] += ways
                                    One => if ca == Deferred and na != Deferred: next[nd][On, hn] += ways
                                    Any => if ca == Deferred and na != Deferred and e == 1: next[nd][Deferred, hn] += ways
                            else:
                                let ha = if ca != Off or n_na != Off: On else: Off
                                next[nd][ha, hn] += ways
                                if ca == Off and e == 1 and d0 == 0: next[nd][On, hn] -= ways
        cur = next
    dp[u] = cur

print([row[a, r] for row in dp[V.first] for a in [Off, On] for r in Bool].sum())
