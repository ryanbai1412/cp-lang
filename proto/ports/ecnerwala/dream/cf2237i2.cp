// Codeforces 2237 I2: Tree DP over a rooted tree with edge labels '0', '1', '?'.
// At each vertex u, `dp[u]` counts valid labellings by `State(depth, active, resolved)`:
//   - `depth`: max 1-edge depth among processed children,
//   - `active` in {Off, On, Deferred}: whether an open 0-chain reaches u;
//     `Deferred` carries the inclusion-exclusion state for an open chain whose
//     closing edge is decided at an ancestor:
//       - across a fixed `Zero` edge, only a `Deferred` child stays `Deferred`,
//       - across a fixed `One` edge, a `Deferred` parent closes to `On`,
//       - across an `Any` ('?') edge, a `Deferred` parent stays `Deferred` on `Edge.One`.
//   - `resolved`: whether a closed chain already sits strictly deepest.
//
// Lifting a child across `edge` closes an active chain into a depth-1 resolved
// chain on `Edge.One`, otherwise adds `edge` to `depth`. Merging `parent` with
// `lifted` obeys four rules:
//   1. at most one active chain and at most one resolved chain,
//   2. an active parent cannot merge across `Edge.Zero`,
//   3. a resolved chain must sit strictly deeper than the other side,
//   4. an active lifted child requires `parent.depth == 0`, and two `Deferred`
//      states cannot meet.
// When `parent` is `(depth: 0, active: Off)` and `edge == Edge.One`, merging
// also subtracts `ways` from `Active.On` (inclusion-exclusion: starting a fresh
// `On` chain at u was already counted by the `Off` baseline). At the root,
// `Deferred` states are excluded because no parent edge can close them.

use std.tree.Rooted

type Z = Mod<1_000_000_007>
enum Label: Zero | One | Any
enum Edge: Zero | One
enum Active: Off | On | Deferred

struct State(depth: Int, active: Active, resolved: Bool)

fn allowed_edges(lbl: Label) -> List<Edge> = match lbl:
    Label.Zero => [Edge.Zero]
    Label.One => [Edge.One]
    Label.Any => [Edge.Zero, Edge.One]

fn lift(child: State, edge: Edge) -> State:
    if edge == Edge.One and child.active != Active.Off:
        State(1, Active.Off, true)
    else:
        State(child.depth + if edge == Edge.One: 1 else: 0, child.active, child.resolved)

fn can_merge(parent: State, lifted: State, child_active: Active, edge: Edge) -> Bool:
    let one_active = parent.active == Active.Off or lifted.active == Active.Off
    let one_resolved = not (parent.resolved and lifted.resolved)
    let edge_ok = parent.active == Active.Off or edge == Edge.One
    let resolved_deepest = (
        not lifted.resolved or parent.depth <= lifted.depth
    ) and (not parent.resolved or parent.depth > lifted.depth)
    let active_clean = (lifted.active == Active.Off or parent.depth == 0) and not (
        parent.active == Active.Deferred and child_active == Active.Deferred
    )
    one_active and one_resolved and edge_ok and resolved_deepest and active_clean

@!multitest
let n: 1..=500 = read()
domain V = 1..=n
let label_str: Str = read()
let label: [V]Label = [V] v => if v == V.first:
    Label.Zero
else:
    match label_str[v.ord - 1]:
        '0' => Label.Zero
        '1' => Label.One
        _ => Label.Any

var par: [V]?V = none
for u in V:
    let deg: Int = read()
    repeat deg: par[read<V>()] = u
let t = Rooted.from_parents(par)

var dp: [V]List<[Active, Bool]Z> = []
for u in t.Pre.rev:
    var cur: List<[Active, Bool]Z> = [
        [Active, Bool] (a, r) => if not r and a != Active.On: Z(1) else: Z(0)
    ]
    for c in t.children(u):
        let child_dp = dp[c]
        var next: List<[Active, Bool]Z> = [
            [Active, Bool] (_, _) => Z(0) for _ in 0..max(cur.len, child_dp.len + 1)
        ]
        for (d0, row0) in cur.items:
            for ca in Active, cr in Bool if row0[ca, cr] != 0:
                let parent = State(d0, ca, cr)
                for (d1, row1) in child_dp.items:
                    for na in Active, nr in Bool if row1[na, nr] != 0:
                        let child = State(d1, na, nr)
                        let ways = row0[ca, cr] * row1[na, nr]
                        for edge in allowed_edges(label[c]):
                            let lifted = lift(child, edge)
                            if not can_merge(parent, lifted, na, edge): continue
                            let nd = max(parent.depth, lifted.depth)
                            let nr_out = parent.resolved or lifted.resolved
                            if ca == Active.Deferred or na == Active.Deferred:
                                match label[c]:
                                    Label.Zero =>
                                        if na == Active.Deferred: next[nd][
                                            Active.Deferred,
                                            nr_out,
                                        ] += ways
                                    Label.One =>
                                        if ca == Active.Deferred and na != Active.Deferred: next[nd][
                                            Active.On,
                                            nr_out,
                                        ] += ways
                                    Label.Any =>
                                        if ca == Active.Deferred and na != Active.Deferred and edge == Edge.One: next[nd][
                                            Active.Deferred,
                                            nr_out,
                                        ] += ways
                            else:
                                let na_out = if ca != Active.Off or lifted.active != Active.Off: Active.On else: Active.Off
                                next[nd][na_out, nr_out] += ways
                                if ca == Active.Off and edge == Edge.One and parent.depth == 0:
                                    next[nd][Active.On, nr_out] -= ways
        cur = next
    dp[u] = cur

// Exclude Active.Deferred at the root: no parent edge exists to close it.
print([row[a, r] for row in dp[V.first] for a in [Active.Off, Active.On] for r in Bool].sum())
