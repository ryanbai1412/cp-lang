// Codeforces 2062 E2: The Game (Hard Version).
// Root at 1 and flatten by preorder so each subtree is a range. A node v wins
// iff the max weight mx outside sub(v) exceeds w[v], and every node u with
// w[u] > w[v] is either inside sub(v), attains mx outside sub(v), or is a
// root-to-leaf path-max ancestor covering all mx-nodes outside sub(v).
struct MaxInfo<P>:
    w: Int
    lo: ?P
    hi: ?P
    cnt: Int

fn combine<P>(a: MaxInfo<P>, b: MaxInfo<P>) -> MaxInfo<P>:
    if a.w > b.w: a
    elif b.w > a.w: b
    elif a.w == 0: a
    else: MaxInfo(a.w, a.lo, b.hi, a.cnt + b.cnt)

@!multitest
let n: 1..=400_000 = read()
domain V = 1..=n
domain W = 1..=n
let w: [V]W = read()
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted.new(Graph.undirected(edges), V.first)

var best_w: [V]Int = 0
var best_v: [V]?V = none
var is_path_max: [V]Bool = false
var prev_max: [V]?V = none
for p in t.Pre:
    let v = t.at[p]
    let pw = if let par = t.par[v]: best_w[par] else: 0
    let pv: ?V = if let par = t.par[v]: best_v[par] else: none
    if w[v].ord + 1 >= pw:
        is_path_max[v] = true
        prev_max[v] = pv
        best_w[v] = w[v].ord + 1
        best_v[v] = v
    else:
        best_w[v] = pw
        best_v[v] = pv

let empty_max = MaxInfo<t.Pre>(0, none, none, 0)
var pref: [0..=n]MaxInfo<t.Pre> = empty_max
for p in t.Pre:
    let cur = MaxInfo<t.Pre>(w[t.at[p]].ord + 1, p, p, 1)
    pref[p.ord + 1] = combine(pref[p.ord], cur)

var suff: [0..=n]MaxInfo<t.Pre> = empty_max
for p in t.Pre.rev:
    let cur = MaxInfo<t.Pre>(w[t.at[p]].ord + 1, p, p, 1)
    suff[p.ord] = combine(cur, suff[p.ord + 1])

var by_w: [W]List<V> = []
for v in V: by_w[w[v]].push(v)

var sub_bit = Fenwick<t.Pre, Int>.new()
var anc_bit = Fenwick<t.Pre, Int>.new()
var winners: List<V> = []
var heavier = 0

for weight in W.rev:
    let w_val = weight.ord + 1
    for v in by_w[weight]:
        let r = t.sub(v)
        let mx = combine(pref[r.lo.ord], suff[r.hi.ord])
        if mx.w > w_val:
            var covered = sub_bit.sum(r) + mx.cnt
            if let lp = prev_max[t.at[mx.lo!]]:
                if mx.hi! in t.sub(lp):
                    covered += anc_bit.prefix(t.pos(lp).r)
            if covered == heavier:
                winners.push(v)
    for v in by_w[weight]:
        if is_path_max[v]:
            anc_bit.add(t.pos(v), 1)
            if let after = t.pos(v) + t.size(v):
                anc_bit.add(after, -1)
        sub_bit.add(t.pos(v), 1)
        heavier += 1

winners.sort()
if winners.is_empty:
    print(0)
else:
    print(winners.len, winners)
