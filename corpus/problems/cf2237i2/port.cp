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
type Table = [Active, Bool]Z

fn parse_label(ch: Char) -> Label = match ch:
    '0' => Label.Zero
    '1' => Label.One
    _ => Label.Any

fn lift(child: State, edge: Edge) -> State:
    if edge == Edge.One and child.active != Active.Off:
        State(1, Active.Off, true)
    else:
        State(child.depth + if edge == Edge.One: 1 else: 0, child.active, child.resolved)

fn merge_edge(parent: State, child: State, lbl: Label, edge: Edge, w: Z, mut out: List<Table>):
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
                add(mut out, State(nd, Active.Deferred, nr), w)
        (Active.Deferred, _) =>
            if lbl == Label.One:
                add(mut out, State(nd, Active.On, nr), w)
            elif lbl == Label.Any and edge == Edge.One:
                add(mut out, State(nd, Active.Deferred, nr), w)
        _ =>
            let na = if parent.active != Active.Off or lifted.active != Active.Off: Active.On else: Active.Off
            add(mut out, State(nd, na, nr), w)
            if parent.active == Active.Off and edge == Edge.One and parent.depth == 0:
                add(mut out, State(nd, Active.On, nr), -w)

fn add(mut out: List<Table>, s: State, w: Z):
    out[s.depth][s.active, s.resolved] += w

fn merge(parent: State, child: State, lbl: Label, w: Z, mut out: List<Table>):
    match lbl:
        Label.Zero => merge_edge(parent, child, lbl, Edge.Zero, w, mut out)
        Label.One => merge_edge(parent, child, lbl, Edge.One, w, mut out)
        Label.Any =>
            merge_edge(parent, child, lbl, Edge.Zero, w, mut out)
            merge_edge(parent, child, lbl, Edge.One, w, mut out)

@!multitest
let n: 2..=3000 = read()
domain V = 1..=n
let label_str: Str = read()
let label: [V]Label = [V] v => if v == V.first!: Label.Zero else: parse_label(label_str[v.ord - 1])

var kids: [V]List<V> = [V] _ => []
var par: [V]?V = None
for u in V:
    repeat read<Int>():
        let c = read<V>()
        kids[u].push(c)
        par[c] = Some(u)
let t = Rooted.from_parents(par)

// `dp[v][depth]` holds the counts of each (active, resolved) state at that depth.
var dp: [V]List<Table> = [V] _ => []
let zero: Table = Z(0)
for p in t.Pre.rev:
    let u = t.at[p]
    var base: Table = Z(0)
    base[Active.Off, false] = Z(1)
    base[Active.Deferred, false] = Z(1)
    var cur: List<Table> = [base]
    for c in kids[u]:
        var next: List<Table> = [zero.clone() for _ in 0..max(cur.len, dp[c].len + 1)]
        for d0 in 0..cur.len:
            for pa in Active:
                for pr in Bool if cur[d0][pa, pr] != Z(0):
                    for d1 in 0..dp[c].len:
                        for ca in Active:
                            for cr in Bool if dp[c][d1][ca, cr] != Z(0):
                                merge(
                                    State(d0, pa, pr),
                                    State(d1, ca, cr),
                                    label[c],
                                    cur[d0][pa, pr] * dp[c][d1][ca, cr],
                                    mut next,
                                )
        cur = next
        dp[c] = []
    dp[u] = cur

// Exclude Active.Deferred at the root: no parent edge exists to close it.
print(
    [row[a, r] for row in dp[V.first!] for a in Active if a != Active.Deferred for r in Bool].sum()
)
