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
// Validity rules (why a merge of `parent` and `lifted = lift(child, edge)` holds):
//   - At most one active open 0-chain (`one_active`) and at most one closed
//     deepest chain (`one_resolved`), because the tree holds a single target path.
//   - An active parent chain can only merge across `Edge.One` (`edge_ok`); a 0-edge
//     would branch a second open 0-chain out of `u`.
//   - A closed chain must sit strictly deeper than all sibling 1-depths
//     (`resolved_deepest`), and an active lifted child requires `parent.depth == 0`
//     so no sibling 1-edge out-depths it before it closes.
//   - Two `Deferred` chains cannot both wait for the same ancestor edge.
//
// Inclusion-exclusion `-1`: when `parent = (depth: 0, active: Off)` merges across
// `Edge.One`, the labelling with `active: On` at `u` is counted both by keeping
// `parent` open (`Deferred` closing at `u`) and by starting the chain at `u`;
// subtracting `1` cancels that double count. At the root, `Deferred` states are
// excluded because no parent edge can close them.

type Z = Mod<1_000_000_007>
enum Label: Zero | One | Any
enum Edge: Zero | One
enum Active: Off | On | Deferred

struct State(depth: Int, active: Active, resolved: Bool)

fn parse_label(ch: Char) -> Label = match ch:
    '0' => Label.Zero
    '1' => Label.One
    _ => Label.Any

fn lift(child: State, edge: Edge) -> State:
    if edge == Edge.One and child.active != Active.Off:
        State(1, Active.Off, true)
    else:
        State(child.depth + if edge == Edge.One: 1 else: 0, child.active, child.resolved)

fn merge_edge(parent: State, child: State, lbl: Label, edge: Edge, mut out: List<(State, Int)>):
    let lifted = lift(child, edge)
    let one_active = parent.active == Active.Off or lifted.active == Active.Off
    let one_resolved = not (parent.resolved and lifted.resolved)
    let edge_ok = parent.active == Active.Off or edge == Edge.One
    let resolved_deepest = (
        not lifted.resolved or parent.depth <= lifted.depth
    ) and (not parent.resolved or parent.depth > lifted.depth)
    let active_clean = lifted.active == Active.Off or parent.depth == 0
    if not (one_active and one_resolved and edge_ok and resolved_deepest and active_clean):
        return
    let nd = max(parent.depth, lifted.depth)
    let nr = parent.resolved or lifted.resolved
    match (parent.active, child.active):
        (Active.Deferred, Active.Deferred) => return
        (_, Active.Deferred) =>
            if lbl == Label.Zero:
                out.push((State(nd, Active.Deferred, nr), 1))
        (Active.Deferred, _) =>
            if lbl == Label.One:
                out.push((State(nd, Active.On, nr), 1))
            elif lbl == Label.Any and edge == Edge.One:
                out.push((State(nd, Active.Deferred, nr), 1))
        _ =>
            let na = if parent.active != Active.Off or lifted.active != Active.Off: Active.On else: Active.Off
            out.push((State(nd, na, nr), 1))
            if parent.active == Active.Off and edge == Edge.One and parent.depth == 0:
                out.push((State(nd, Active.On, nr), -1))

fn merge(parent: State, child: State, lbl: Label) -> List<(State, Int)>:
    var out: List<(State, Int)> = []
    match lbl:
        Label.Zero => merge_edge(parent, child, lbl, Edge.Zero, mut out)
        Label.One => merge_edge(parent, child, lbl, Edge.One, mut out)
        Label.Any =>
            merge_edge(parent, child, lbl, Edge.Zero, mut out)
            merge_edge(parent, child, lbl, Edge.One, mut out)
    out

@!multitest
let n: 2..=3000 = read()
domain V = 1..=n
let label_str: Str = read()
let label: [V]Label = [V] v => if v == V.first: Label.Zero else: parse_label(label_str[v.ord - 1])

var par: [V]?V = None
var kids: [V]List<V> = [V] _ => []
for u in V:
    repeat read<Int>():
        let c = read<V>()
        kids[u].push(c)
        par[c] = u
let t = Rooted.from_parents(par)

var dp: [V]Map<State, Z> = [V] _ => Map<State, Z>()
for p in t.Pre.rev:
    let u = t.at[p]
    var cur: Map<State, Z> = Map()
    cur[State(0, Active.Off, false)] = Z(1)
    cur[State(0, Active.Deferred, false)] = Z(1)
    for c in kids[u]:
        var next: Map<State, Z> = Map()
        for (parent, w0) in cur.items:
            for (child, w1) in dp[c].items:
                for (s, sign) in merge(parent, child, label[c]):
                    next[s] = (next.get(s) ?? Z(0)) + sign * w0 * w1
        cur = next
    dp[u] = cur

// Exclude Active.Deferred at the root: no parent edge exists to close it.
print([w for (s, w) in dp[V.first!].items if s.active != Active.Deferred].sum())
