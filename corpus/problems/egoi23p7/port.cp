// EGOI '23 P7 (Find the Tree): component contraction + complement-graph DFS.
// Runnable on proto/cpc today.
// original: corpus/submissions/5678057.cpp
let n: 1..=3e5 = read()
let m1: 0..=3e5 = read()
let m2: 0..=3e5 = read()
domain V = 0..n

var req_u: List<V> = []
var req_v: List<V> = []
var adj1: [V]List<V> = []
var ideg: [V]Int = 0
for _ in 0..m1:
    let child: V = read()
    let parent: V = read()
    req_u.push(parent)
    req_v.push(child)
    adj1[parent].push(child)
    ideg[child] += 1

var forb_in: [V]Set<V> = []
for _ in 0..m2:
    let child: V = read()
    let parent: V = read()
    forb_in[child].insert(parent)

var vis: [V]Bool = false
var has_cyc = false
var ord: List<V> = []
fn dfs_req(u: V):
    if vis[u]:
        has_cyc = true
        return
    vis[u] = true
    for v in adj1[u]: dfs_req(v)
    ord.push(u)

fn solve():
    var rep: List<V> = []
    var comps: List<List<V>> = []
    var comp_of: [V]Int = 0
    for u in V:
        if ideg[u] != 0: continue
        ord = []
        dfs_req(u)
        let cid = rep.len
        rep.push(u)
        comps.push(ord)
        for x in ord: comp_of[x] = cid

    if has_cyc:
        print("NO")
        return
    for u in V:
        if not vis[u]:
            print("NO")
            return

    let ccnt = rep.len
    domain Comp = 0..ccnt
    var cc_adj: [Comp]List<Comp> = []
    for c in Comp:
        var cnt = Map<Int, Int>()
        for u in forb_in[rep[c.idx]]:
            cnt[comp_of[u]] += 1
        for (k, v) in cnt:
            if v == comps[k].len:
                cc_adj[Comp(k)].push(c)

    var cvis: [Comp]Bool = false
    var to_vis = Set<Comp>()
    for c in Comp: to_vis.insert(c)
    var comp_u: List<Comp> = []
    var comp_v: List<Comp> = []

    fn dfs_comp(u: Comp, p: ?Comp):
        if let par = p:
            comp_u.push(par)
            comp_v.push(u)
        if cvis[u]: return
        cvis[u] = true
        var can_vis = Set<Comp>()
        var cant_vis = Set<Comp>()
        for c in to_vis: can_vis.insert(c)
        to_vis = Set<Comp>()
        for v in cc_adj[u]:
            if v in can_vis:
                can_vis.remove(v)
                cant_vis.insert(v)
        for c in cant_vis: to_vis.insert(c)
        for v in can_vis: dfs_comp(v, u)

    for c in Comp:
        if c in to_vis:
            to_vis.remove(c)
            dfs_comp(c, None)
            to_vis.insert(c)

    if comp_u.len != ccnt - 1:
        print("NO")
        return

    for i in 0..m1:
        print(req_v[i].idx, req_u[i].idx)
    for i in 0..comp_u.len:
        let u = comp_u[i]
        let v = comp_v[i]
        let rv = rep[v.idx]
        for a in comps[u.idx]:
            if a not in forb_in[rv]:
                print(rv.idx, a.idx)
                break

solve()
