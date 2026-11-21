// Codeforces 2237 I1: bottom-up tree DP counting canonical valid 0/1 edge
// assignments matching a template string ('0', '1', or '?').
//
// At each vertex u, dp[u][d][st] counts valid assignments in u's subtree with
// max chain depth d and cut state st = (active, resolved):
//   - active: an open active chain reaches u (at most one, cannot follow a
//     0-edge, and cannot coexist with depth > 0).
//   - resolved: a resolved chain already sits strictly deepest among processed
//     children (at most one, strictly deeper than all other children).
// Crossing a 1-edge resolves an active chain to (active=false, resolved=true,
// depth=1); otherwise the edge adds e to depth. When ca=false, e=1, and d0=0,
// subtracting ways from (active=true, resolved=hn) removes the duplicate
// non-canonical representation.

use std.tree.Rooted

type Z = Mod<1_000_000_007>
domain State = 0..4

fn make_state(active: Bool, resolved: Bool) -> State =
    State((if active: 2 else: 0) + (if resolved: 1 else: 0))

fn is_active(st: State) -> Bool = st.ord >= 2
fn is_resolved(st: State) -> Bool = st.ord % 2 == 1

fn matches_edge(ch: Char, e: Int) -> Bool =
    ch == '?' or (e == 0 and ch == '0') or (e == 1 and ch == '1')

fn can_merge(d0: Int, ca: Bool, cn: Bool, nd1: Int, n_na: Bool, n_nn: Bool, e: Int) -> Bool:
    if ca and (n_na or e == 0): return false
    if cn and (n_nn or d0 <= nd1): return false
    if n_nn and d0 > nd1: return false
    if n_na and e == 0 and d0 > 0: return false
    true

@!multitest
let n: 1..=500 = read()
domain V = 1..=n
let raw_s: Str = read()
let label: [V]Char = [V] v => if v == V.first: '0' else: raw_s[v.ord - 1]

var par: [V]?V = none
var children: [V]List<V> = []
for u in V:
    let deg: 0..n = read()
    for _ in 0..deg:
        let c: V = read()
        par[c] = u
        children[u].push(c)
let t = Rooted.from_parents(par)

var dp: [V]List<[State]Z> = []
for p in t.Pre.rev:
    let u = t.at[p]
    var init_row: [State]Z = 0
    init_row[make_state(false, false)] = 1
    var cur: List<[State]Z> = [init_row]
    for c in children[u]:
        let child = dp[c]
        let nh = max(cur.len, child.len + 1)
        var next: List<[State]Z> = [[State] _ => Z(0) for _ in 0..nh]
        let ch = label[c]
        for (d0, row0) in cur.items:
            for cs in State:
                let v0 = row0[cs]
                if v0 == 0: continue
                let (ca, cn) = (is_active(cs), is_resolved(cs))
                for (d1, row1) in child.items:
                    for ns in State:
                        let v1 = row1[ns]
                        if v1 == 0: continue
                        let ways = v0 * v1
                        let (na, nn) = (is_active(ns), is_resolved(ns))
                        for e in 0..=1:
                            if not matches_edge(ch, e): continue
                            let (
                                n_na,
                                n_nn,
                                nd1,
                            ) = if e == 1 and na: (false, true, 1) else: (na, nn, d1 + e)
                            if not can_merge(d0, ca, cn, nd1, n_na, n_nn, e): continue
                            let nd = max(d0, nd1)
                            let ha = ca or n_na
                            let hn = cn or n_nn
                            next[nd][make_state(ha, hn)] += ways
                            if not ca and e == 1 and d0 == 0:
                                next[nd][make_state(true, hn)] -= ways
        cur = next
    dp[u] = cur

print([row[st] for row in dp[V.first] for st in State].sum())
