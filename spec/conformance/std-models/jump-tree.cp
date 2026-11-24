// expect: ok, output "ok"
// input: "1 300\n"
// model: JumpTree
// model: Lift
// A JumpTree grown by seeded random add (a lone vertex under any other) against climbing its
// parent array: parent, depth, root_of, ancestor, lca, jump_while, dist and jump; and a Lift over
// a seeded random weighted forest (rebuilt every 20 ops) with climb, ancestor, lca, fold and
// climb_while, plus Lift.of on the same links. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..14
var rng = Rng(seed0)
fn climb(par: [V]?V, v: V) -> List<V>:
    var out: List<V> = [v]
    while let p = par[out[out.len - 1]]: out.push(p)
    out
fn bpath(par: [V]?V, a: V, b: V) -> ?List<V>:
    let (ca, cb) = (climb(par, a), climb(par, b))
    if ca[ca.len - 1] != cb[cb.len - 1]: return None
    var i = 0
    while not cb.contains(ca[i]): i += 1
    var j = 0
    while cb[j] != ca[i]: j += 1
    Some(ca[..=i] + cb[..j].rev.collect())
var j = JumpTree<V>()
var par: [V]?V = None
var lone: [V]Bool = true
var lp: [V]?(V, Int) = None
var bad = ""
for k in 0..n:
    let free = [v for v in V if lone[v]]
    if free.len == 0 or rng.int(0, 30) == 0:
        j = JumpTree<V>()
        par = [V] _ => None
        lone = [V] _ => true
    else:
        let v = free[rng.int(0, free.len - 1)]
        var p = V(rng.int(0, 13))
        if p == v: p = V((p.ord + 1) % 14)
        j.add(v, p)
        par[v] = Some(p)
        lone[v] = false
        lone[p] = false
    if k % 20 == 0:
        var perm = [V(i) for i in 0..14]
        rng.shuffle(mut perm)
        lp = [V] _ => None
        for i in 1..14:
            if rng.int(0, 5) > 0: lp[perm[i]] = Some((perm[rng.int(0, i - 1)], rng.int(0, 6)))
    let (a, b) = (V(rng.int(0, 13)), V(rng.int(0, 13)))
    let ca = climb(par, a)
    let pab = bpath(par, a, b)
    let kk = rng.int(0, 14)
    let lim = rng.int(0, 6)
    var want_jw: ?V = None
    for u in ca:
        if climb(par, u).len - 1 >= lim: want_jw = Some(u) else: break
    let want_lca: ?V = if let q = pab: q.find(x => ca.contains(x) and climb(par, b).contains(x)) else: None
    if j.parent(a) != par[a] or j.depth(a) != ca.len - 1 or j.root_of(a) != ca[ca.len - 1]: bad = "op {k}: parent/depth/root_of {a}"
    elif j.ancestor(a, kk) != (if kk < ca.len: Some(ca[kk]) else: None) or j.lca(a, b) != want_lca: bad = "op {k}: ancestor/lca {a} {b}"
    elif j.jump_while(a, u => j.depth(u) >= lim) != want_jw: bad = "op {k}: jump_while {a} {lim}"
    elif j.dist(a, b) != pab.map(q => q.len - 1): bad = "op {k}: dist {a} {b}"
    elif j.jump(a, b, kk) != (if let q = pab: (if kk < q.len: Some(q[kk]) else: None) else: None): bad = "op {k}: jump {a} {b} {kk}"
    if bad != "":
        break
    let up = [V] v => lp[v].map(e => e.0)
    let l = Lift(lp, op: (x, y) => x + y, identity: 0)
    let lo = Lift<V, ()>.of(v => up[v])
    let cl = climb(up, a)
    let lq = bpath(up, a, b)
    let lw: ?V = if let q = lq: q.find(x => cl.contains(x) and climb(up, b).contains(x)) else: None
    let u = cl[rng.int(0, cl.len - 1)]
    var fsum = 0
    var x = a
    while x != u:
        let (p, w) = lp[x]!
        fsum += w
        x = p
    var steps = 0
    while steps + 1 < cl.len and climb(up, cl[steps + 1]).len - 1 >= lim: steps += 1
    var top = 0
    var acc = 0
    while top + 1 < cl.len and acc + lp[cl[top]]!.1 <= lim + 3:
        acc += lp[cl[top]]!.1
        top += 1
    if l.parent(a) != up[a] or l.depth(a) != cl.len - 1 or lo.depth(a) != cl.len - 1: bad = "op {k}: lift parent/depth {a}"
    elif l.ancestor(a, kk) != (if kk < cl.len: Some(cl[kk]) else: None) or lo.ancestor(a, kk) != l.ancestor(a, kk): bad = "op {k}: lift ancestor {a} {kk}"
    elif l.lca(a, b) != lw: bad = "op {k}: lift lca {a} {b}"
    elif l.fold(a, up_to: u) != fsum: bad = "op {k}: lift fold {a} {u}"
    elif l.climb(a, y => l.depth(y) >= lim) != (steps: steps, at: cl[steps]): bad = "op {k}: lift climb {a} {lim}"
    elif l.climb_while(a, s => s <= lim + 3) != cl[top]: bad = "op {k}: lift climb_while {a} {lim}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
