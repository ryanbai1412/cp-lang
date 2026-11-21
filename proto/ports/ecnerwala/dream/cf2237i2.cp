// Codeforces 2237 I2: Tree DP over a rooted tree with edge labels '0', '1', '?'.
// At each vertex u, `dp[u]` counts valid labellings keyed by
// `State(depth, active, resolved)`:
//   - `depth`: max 1-edge depth among processed children,
//   - `active` in {Off, On, Deferred}: whether an open 0-chain reaches u;
//     `Deferred` carries the inclusion-exclusion state for an open chain whose
//     closing edge is decided at an ancestor:
//       - across a fixed `Zero` edge, only a `Deferred` child stays `Deferred`,
//       - across a fixed `One` edge, a `Deferred` parent closes to `On`,
//       - across an `Any` ('?') edge, a `Deferred` parent stays `Deferred` on `Edge.One`.
//   - `resolved`: whether a closed chain already sits strictly deepest.
//
// `merge(parent, child, lbl)` owns lifting a child across each allowed edge,
// checking the four validity rules (at most one active, at most one resolved,
// active parent only across `Edge.One`, resolved strictly deepest, active
// lifted child requires `parent.depth == 0`, no two `Deferred` meeting), and
// emitting `(next_state, sign)` including the `-1` inclusion-exclusion term
// when `(depth: 0, active: Off)` merges across `Edge.One`. At the root,
// `Deferred` states are excluded because no parent edge can close them.

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

fn merge(parent: State, child: State, lbl: Label) -> List<(State, Int)>:
    var out: List<(State, Int)> = []
    for edge in allowed_edges(lbl):
        let lifted = lift(child, edge)
        let one_active = parent.active == Active.Off or lifted.active == Active.Off
        let one_resolved = not (parent.resolved and lifted.resolved)
        let edge_ok = parent.active == Active.Off or edge == Edge.One
        let resolved_deepest = (
            not lifted.resolved or parent.depth <= lifted.depth
        ) and (not parent.resolved or parent.depth > lifted.depth)
        let active_clean = (lifted.active == Active.Off or parent.depth == 0) and not (
            parent.active == Active.Deferred and child.active == Active.Deferred
        )
        if not (one_active and one_resolved and edge_ok and resolved_deepest and active_clean):
            continue
        let nd = max(parent.depth, lifted.depth)
        let nr = parent.resolved or lifted.resolved
        if parent.active == Active.Deferred or child.active == Active.Deferred:
            match lbl:
                Label.Zero =>
                    if child.active == Active.Deferred:
                        out.push((State(nd, Active.Deferred, nr), 1))
                Label.One =>
                    if parent.active == Active.Deferred and child.active != Active.Deferred:
                        out.push((State(nd, Active.On, nr), 1))
                Label.Any =>
                    if parent.active == Active.Deferred and child.active != Active.Deferred and edge == Edge.One:
                        out.push((State(nd, Active.Deferred, nr), 1))
        else:
            let na = if parent.active != Active.Off or lifted.active != Active.Off: Active.On else: Active.Off
            out.push((State(nd, na, nr), 1))
            if parent.active == Active.Off and edge == Edge.One and parent.depth == 0:
                out.push((State(nd, Active.On, nr), -1))
    out

@!multitest
let n: 1..=500 = read()
domain V = 1..=n
let label: [V]Label = read(char_map: {'0': Label.Zero, '1': Label.One, '?': Label.Any})

var par: [V]?V = none
for u in V:
    repeat read<Int>(): par[read<V>()] = u
let t = Rooted.from_parents(par)

var dp: [V]Map<State, Z> = Map()
for u in t.Pre.rev:
    var cur: Map<State, Z> = {State(0, Active.Off, false): 1, State(0, Active.Deferred, false): 1}
    for c in t.children(u):
        var next: Map<State, Z> = Map()
        for (parent, w0) in cur.items:
            for (child, w1) in dp[c].items:
                for (s, sign) in merge(parent, child, label[c]):
                    next[s] += sign * w0 * w1
        cur = next
    dp[u] = cur

// Exclude Active.Deferred at the root: no parent edge exists to close it.
print([w for (s, w) in dp[V.first].items if s.active != Active.Deferred].sum())
