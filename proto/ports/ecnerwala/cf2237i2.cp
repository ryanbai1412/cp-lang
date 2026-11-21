// Codeforces 2237 I2.
// Tree DP over a rooted tree whose edges carry '0', '1', or '?'.
// At each vertex, `dp[u][depth][active, resolved]` counts ways where:
//   - `depth` is the max depth of a resolved/active chain in processed children,
//   - `active` in {OFF=0, ON=1, DEF=2} tracks whether an open active path reaches u
//     (`DEF` carries the inclusion-exclusion adjustment on '0'/'1'/'?' edges),
//   - `resolved` in {0, 1} tracks whether a resolved chain already sits strictly deepest.
use std.tree.Rooted

type Z = Mod<1_000_000_007>
enum Label: Zero | One | Any

const OFF = 0
const ON = 1
const DEF = 2

fn can_merge(d0: Int, ca: Int, cn: Int, nd1: Int, n_na: Int, n_nn: Int, na: Int, e: Int) -> Bool:
    if ca != OFF and n_na != OFF: return false
    if cn != 0 and n_nn != 0: return false
    if ca != OFF and e == 0: return false
    if (n_nn != 0 and d0 > nd1) or (cn != 0 and d0 <= nd1): return false
    if (n_na != OFF and d0 > 0) or (ca == DEF and na == DEF): return false
    true

@!multitest
let n: 1..=500 = read()
domain V = 1..=n
let label_str: Str = read()
var label: [V]Label = Label.Zero
for v in V if v != V.first:
    label[v] = match label_str[v.ord - 1]:
        '0' => Label.Zero
        '1' => Label.One
        _ => Label.Any

var par: [V]?V = none
for u in V:
    let deg: Int = read()
    repeat deg: par[read<V>()] = u
let t = Rooted.from_parents(par)

var dp: [V]List<[0..3, 0..2]Z> = []
for p in t.Pre.rev:
    let u = t.at[p]
    var init_row: [0..3, 0..2]Z = 0
    init_row[OFF, 0] = 1
    init_row[DEF, 0] = 1
    var cur: List<[0..3, 0..2]Z> = [init_row]
    for c in t.children(u):
        let child = dp[c]
        let nh = max(cur.len, child.len + 1)
        var zero_row: [0..3, 0..2]Z = 0
        var next: List<[0..3, 0..2]Z> = [zero_row for _ in 0..nh]
        for (d0, row0) in cur.items:
            for ca in 0..3:
                for cn in 0..2 if row0[ca, cn] != 0:
                    let vc = row0[ca, cn]
                    for (d1, row1) in child.items:
                        for na in 0..3:
                            for nn in 0..2 if row1[na, nn] != 0:
                                let ways = vc * row1[na, nn]
                                for e in 0..2:
                                    if (
                                        label[c] == Label.Zero and e == 1
                                    ) or (label[c] == Label.One and e == 0): continue
                                    let (
                                        n_na,
                                        n_nn,
                                        nd1,
                                    ) = if e == 1 and na != OFF: (OFF, 1, 1) else: (na, nn, d1 + e)
                                    if not can_merge(d0, ca, cn, nd1, n_na, n_nn, na, e): continue
                                    let nd = max(d0, nd1)
                                    let hn = if cn != 0 or n_nn != 0: 1 else: 0
                                    if ca == DEF or na == DEF:
                                        match label[c]:
                                            Label.Zero =>
                                                if na == DEF: next[nd][DEF, hn] += ways
                                            Label.One =>
                                                if ca == DEF and na != DEF: next[nd][ON, hn] += ways
                                            Label.Any =>
                                                if ca == DEF and na != DEF and e == 1: next[nd][
                                                    DEF,
                                                    hn,
                                                ] += ways
                                    else:
                                        let ha = if ca != OFF or n_na != OFF: ON else: OFF
                                        next[nd][ha, hn] += ways
                                        if ca == OFF and e == 1 and d0 == 0: next[nd][
                                            ON,
                                            hn,
                                        ] -= ways
        cur = next
    dp[u] = cur

print([row[a, r] for row in dp[V.first] for a in 0..2 for r in 0..2].sum())
