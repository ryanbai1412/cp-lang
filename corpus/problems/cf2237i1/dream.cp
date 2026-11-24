// Codeforces 2237 I1: bottom-up tree DP counting canonical valid 0/1 edge
// assignments matching a template string ('0', '1', or '?').
//
// At each vertex `u`, `dp[u]` counts valid labellings per
// `State(depth, active, resolved)`, packed densely at `slot(state)`:
//   - `depth`: max 1-edge depth among processed children,
//   - `active`: an open 0-chain reaches `u` (at most one, cannot cross a
//     0-edge from `parent`, and cannot coexist with `parent.depth > 0`),
//   - `resolved`: a closed chain already sits strictly deepest among processed
//     children (at most one, strictly deeper than all sibling 1-depths).
//
// Lifting `child` across `Edge.One` closes an active child chain into a depth-1
// resolved chain `State(1, false, true)`; otherwise the edge adds `1` (on `One`)
// or `0` (on `Zero`) to `child.depth`.
//
// Inclusion-exclusion `-1`: when `parent = (depth: 0, active: false)` merges
// across `Edge.One`, the same subtree labelling with `active: true` at `u` is
// reached both by extending the inactive prefix and by starting an active chain
// at `u`; subtracting `1` cancels that duplicate non-canonical representation.

type Z = Mod<1_000_000_007>
enum Label: Zero | One | Any
enum Edge: Zero | One

struct State(depth: Int, active: Bool, resolved: Bool)

fn slot(s: State) -> Int = 4 * s.depth + (if s.active: 2 else: 0) + (if s.resolved: 1 else: 0)
fn state_at(i: Int) -> State = State(i / 4, i % 4 >= 2, i % 2 == 1)

fn parse_label(ch: Char) -> Label = match ch:
    '0' => Label.Zero
    '1' => Label.One
    _ => Label.Any

fn lift(child: State, edge: Edge) -> State:
    if edge == Edge.One and child.active:
        State(1, false, true)
    else:
        State(child.depth + if edge == Edge.One: 1 else: 0, child.active, child.resolved)

fn merge_edge(parent: State, child: State, edge: Edge, w: Z, mut next: List<Z>):
    let lifted = lift(child, edge)
    let one_active = not (parent.active and lifted.active)
    let one_resolved = not (parent.resolved and lifted.resolved)
    let edge_ok = not parent.active or edge == Edge.One
    let resolved_deepest = (
        not lifted.resolved or parent.depth <= lifted.depth
    ) and (not parent.resolved or parent.depth > lifted.depth)
    let active_clean = not (lifted.active and edge == Edge.Zero and parent.depth > 0)
    if not (one_active and one_resolved and edge_ok and resolved_deepest and active_clean):
        return
    let nd = max(parent.depth, lifted.depth)
    let na = parent.active or lifted.active
    let nr = parent.resolved or lifted.resolved
    next[slot(State(nd, na, nr))] += w
    if not parent.active and edge == Edge.One and parent.depth == 0:
        next[slot(State(nd, true, nr))] -= w

fn merge(parent: State, child: State, lbl: Label, w: Z, mut next: List<Z>):
    match lbl:
        Label.Zero => merge_edge(parent, child, Edge.Zero, w, mut next)
        Label.One => merge_edge(parent, child, Edge.One, w, mut next)
        Label.Any =>
            merge_edge(parent, child, Edge.Zero, w, mut next)
            merge_edge(parent, child, Edge.One, w, mut next)

@!multitest
let n: 2..=3000 = read()
domain V = 1..=n
let label_str: Str = read()
let label: [V]Label = [V] v => if v == V.first!: Label.Zero else: parse_label(label_str[v.ord - 1])

// Keep the input child order: sibling tie-breaking is order-sensitive, and Rooted.from_parents discards it.
var par: [V]?V = None
var children: [V]List<V> = []
for u in V:
    repeat read<Int>():
        let c: V = read()
        par[c] = u
        children[u].push(c)
let t = Rooted.from_parents(par)

// A merge deepens the child by at most one edge, so `next` needs one depth (4 slots) more.
var dp: [V]List<Z> = [V] _ => []
for p in t.Pre.rev:
    let u = t.at[p]
    var cur = List.filled(4, Z(0))
    cur[slot(State(0, false, false))] = Z(1)
    for c in children[u]:
        var next = List.filled(max(cur.len, dp[c].len + 4), Z(0))
        for i in 0..cur.len if cur[i] != Z(0):
            for j in 0..dp[c].len if dp[c][j] != Z(0):
                merge(state_at(i), state_at(j), label[c], cur[i] * dp[c][j], mut next)
        dp[c] = []
        cur = next
    dp[u] = cur

print(dp[V.first!].sum())
