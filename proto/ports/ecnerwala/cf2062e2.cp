use std.fenwick.{DualFenwick}
use std.prefix.{prefix_fold, suffix_fold}
use std.seq.{buckets}

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

// top[v] is the deepest path-maximum on root -> v, so v is a path-max iff top[v] == v.
let above: ?V = none
let top = t.down(above, (s, v) => if let u = s, w[u] > w[v]: s else: some(v))

let m_max = Monoid(MaxInfo<t.Pre>(0, none, none, 0), (a, b) => combine(a, b))
let pre_info = [t.Pre] p => MaxInfo<t.Pre>(w[t.at[p]].ord + 1, p, p, 1)
let pref = prefix_fold(m_max, pre_info)
let suff = suffix_fold(m_max, pre_info)
let by_w = buckets(w)

var sub_bit = Fenwick<t.Pre, Int>.new()
var anc_bit = DualFenwick<t.Pre, Int>.new()
var winners: List<V> = []
var heavier = 0

for weight in W.rev:
    let w_val = weight.ord + 1
    for v in by_w[weight]:
        let r = t.sub(v)
        let mx = combine(pref[r.start], suff[r.end])
        if mx.w > w_val:
            var covered = sub_bit.sum(r) + mx.cnt
            let lo = t.at[mx.lo!]
            if top[lo] == lo, let p = t.parent(lo), let lp = top[p], mx.hi! in t.sub(lp):
                covered += anc_bit.get(t.pos(lp))
            if covered == heavier:
                winners.push(v)
    for v in by_w[weight]:
        if top[v] == v:
            anc_bit.add(t.sub(v), 1)
        sub_bit.add(t.pos(v), 1)
        heavier += 1

winners.sort()
if winners.is_empty:
    print(0)
else:
    print(winners.len, winners)
