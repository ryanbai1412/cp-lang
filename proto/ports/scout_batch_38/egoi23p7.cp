// EGOI '23 P7 (Find the Tree): build a rooted spanning arborescence containing
// all m1 required directed edges and none of the m2 forbidden edges, or print NO.
// Component contraction + O((C + m2) log C) complement-graph DFS via Set swap.
// original: corpus/submissions/5678057.cpp
let (n, m1, m2): (1..=3e5, 0..=3e5, 0..=3e5) = read()
domain V = 0..n
// Input gives (child, parent); orient edges parent -> child internally.
let req: [0..m1](child: V, parent: V) = read()
let forb: [0..m2](child: V, parent: V) = read()

fn solve() -> ?List<(V, V)>:
    var adj1: [V]List<V> = []
    var ideg: [V]Int = 0
    for e in req:
        adj1[e.parent].push(e.child)
        ideg[e.child] += 1

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

    var reps_list: List<V> = []
    var comps_list: List<List<V>> = []
    for u in V if ideg[u] == 0:
        ord = []
        dfs_req(u)
        reps_list.push(u)
        comps_list.push(ord)

    if has_cyc or any(not vis[u] for u in V): return none

    let (domain Comp, rep) = reps_list.freeze()
    var comps: [Comp]List<V> = [Comp] c => comps_list[c.to_idx]
    var comp_of: [V]Comp = Comp.first
    for c in Comp:
        for u in comps[c]: comp_of[u] = c

    var forb_in: [V]HashSet<V> = [V] _ => HashSet<V>()
    for e in forb: forb_in[e.child].add(e.parent)

    // Component edge k -> c is completely blocked iff every node in comps[k]
    // is forbidden from connecting to rep[c].
    var cc_adj: [Comp]List<Comp> = []
    for c in Comp:
        var cnt = HashMap<Comp, Int>()
        for u in forb_in[rep[c]]: cnt[comp_of[u]] += 1
        for (k, v) in cnt.items if v == comps[k].len:
            cc_adj[k].push(c)

    // Complement-graph DFS over Comp in O((C + m2) log C).
    var cvis: [Comp]Bool = false
    var to_vis = Set<Comp>()
    for c in Comp: to_vis.add(c)
    var comp_tree: List<(Comp, Comp)> = []

    fn dfs_comp(u: Comp, p: ?Comp):
        if let par = p: comp_tree.push((par, u))
        if cvis[u]: return
        cvis[u] = true
        var can_vis = Set<Comp>()
        var cant_vis = Set<Comp>()
        swap(mut can_vis, mut to_vis)
        // FRICTION 38.11: Set `-`/`&` would be O(|to_vis|) = O(C^2), and
        // Set.remove(v) has no documented Bool return, forcing membership + remove.
        for v in cc_adj[u]:
            if v in can_vis:
                can_vis.remove(v)
                cant_vis.add(v)
        swap(mut cant_vis, mut to_vis)
        for v in can_vis: dfs_comp(v, u)

    for c in Comp:
        if c in to_vis:
            to_vis.remove(c)
            dfs_comp(c, none)
            to_vis.add(c)

    if comp_tree.len != Comp.K - 1: return none

    var ans: List<(V, V)> = [(e.child, e.parent) for e in req]
    for (u, v) in comp_tree:
        let a = first(x for x in comps[u] if x not in forb_in[rep[v]])!
        ans.push((rep[v], a))
    ans

if let ans = solve():
    for (child, parent) in ans: print(child, parent)
else:
    print("NO")
