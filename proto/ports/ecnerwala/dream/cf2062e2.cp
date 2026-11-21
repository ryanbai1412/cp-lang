// Codeforces 2062 E2: The Game (Hard Version) (dream).
// Root at 1 and flatten by preorder so each subtree is a range. A node v wins
// iff the max weight mx outside sub(v) exceeds w[v], and every node u with
// w[u] > w[v] is either inside sub(v), attains mx outside sub(v), or is a
// root-to-leaf path-max ancestor covering all mx-nodes outside sub(v).
with io(base: 1)

struct MaxInfo:
    w: Int
    lo: Int
    hi: Int
    cnt: Int

@!multitest
domain V = read(1..=4e5)
let w: [V]1..=V.count = read()
let t = Rooted(Graph.undirected(read<[V.count - 1](V, V)>()), root: V.first)

// For each path-prefix-max node v, prev_max[v] is its previous path-prefix-max ancestor.
let (is_path_max, prev_max) = t.top_down((best_w: 0, best_v: none), (st, v) =>
    if w[v] >= st.best_w: ((true, st.best_v), (w[v], some(v)))
    else: ((false, none), st)
)

let wp = t.reorder(w)
let pref = prefix_fold(wp, MaxInfo.empty, MaxInfo.add)
let suff = suffix_fold(wp, MaxInfo.empty, MaxInfo.add)
var sub_bit = Fenwick<Pre, Int>()
var anc_bit = Fenwick<Pre, Int>()

var winners: List<V> = []
var heavier = 0
for bucket in V.group(by: v => w[v]).rev:
    for v in bucket:
        let r = t.sub(v)
        let mx = MaxInfo.merge(pref[r.lo], suff[r.hi])
        if mx.w > w[v]:
            var covered = sub_bit.sum(r) + mx.cnt
            if let lp = prev_max[t.at(mx.lo)] where mx.hi in t.sub(lp):
                covered += anc_bit.prefix(t.pos(lp).r)
            if covered == heavier:
                winners.push(v)
    for v in bucket:
        if is_path_max[v]:
            anc_bit.range_add(t.sub(v), 1)
        sub_bit.add(t.pos(v), 1)
        heavier += 1

winners.sort()
print(winners.len, winners)
