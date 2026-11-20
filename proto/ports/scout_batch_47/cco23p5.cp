// CCO '23 P5 — Tree Walk / K-Jump Path (DMOJ: cco23p5)
// original: corpus/submissions/5580402.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Given a weighted tree on n vertices and a jump limit k in {1, 2, 3}, find a
// simple walk starting at vertex 1 whose consecutive vertices are at tree
// distance at most k, maximizing the total weight of visited vertices, and
// output both the maximum weight and the walk.
let (n, k): (1..=2e5, 1..=3) = read()
domain V = 1..=n
let edges: [1..n](V, V) = read()
let w: [V]Int = read()

let g = Graph.undirected(edges)
let tree = Rooted<V>(g, V.first)
var walk: List<V> = []

// Top-k distinct children maximizing callback scores (eliminates -1e18 / -1 sentinels).
fn top_k(ch: [V], lim: Int, score: V -> Int) -> List<(Int, V)>:
    var items: List<(Int, V)> = [(score(v), v) for v in ch]
    sort(mut items, by: x => -x.0)
    if items.len > lim: items[..lim].copy() else: items

fn opt2(ch: [V], f1: V -> Int, f2: V -> Int) -> ?(Int, V, V):
    let (t1, t2) = (top_k(ch, 2, f1), top_k(ch, 2, f2))
    var best: ?(Int, V, V) = none
    for (w1, v1) in t1:
        for (w2, v2) in t2 if v1 != v2:
            best max= (w1 + w2, v1, v2)
    best

fn opt3(ch: [V], f1: V -> Int, f2: V -> Int, f3: V -> Int) -> ?(Int, V, V, V):
    let (t1, t2, t3) = (top_k(ch, 3, f1), top_k(ch, 3, f2), top_k(ch, 3, f3))
    var best: ?(Int, V, V, V) = none
    for (w1, v1) in t1:
        for (w2, v2) in t2 if v1 != v2:
            for (w3, v3) in t3 if v3 != v1 and v3 != v2:
                best max= (w1 + w2 + w3, v1, v2, v3)
    best

@memo
fn dp1(u: V) -> Int:
    w[u] + (max(dp1(v) for v in tree.children(u)) ?? 0)

fn build1(u: V):
    walk.push(u)
    let rem = dp1(u) - w[u]
    if rem == 0: return
    for v in tree.children(u) if dp1(v) == rem:
        build1(v)
        return

// k = 2: dp2(u, me, back) is max weight inside subtree(u), where:
//   me   = whether the walk enters at u itself (vs a child of u),
//   back = whether the walk must finish at u or a child of u to jump out.
@memo
fn dp2(u: V, me: Bool, back: Bool) -> Int:
    let ch = tree.children(u)
    let tot = sum(w[v] for v in ch)
    var best = 0
    if not back:
        for v in ch: best max= dp2(v, false, false)
        if let (val, _, _) = opt2(ch, v => dp2(v, false, true) - w[v], v => dp2(v, true, false) - w[v]):
            best max= tot + val
        if not me:
            if let (val, _, _, _) = opt3(
                ch,
                v => dp2(v, true, true) - w[v],
                v => dp2(v, false, true) - w[v],
                v => dp2(v, true, false) - w[v],
            ):
                best max= tot + val
            if let (val, _, _) = opt2(ch, v => dp2(v, true, true) - w[v], v => dp2(v, false, false) - w[v]):
                best max= tot + val
            for v in ch: best max= tot - w[v] + dp2(v, false, false)
    elif me:
        for v in ch: best max= tot - w[v] + dp2(v, false, true)
    else:
        for v in ch: best max= tot - w[v] + dp2(v, true, true)
    best + w[u]

fn build2(u: V, me: Bool, back: Bool):
    let ch = tree.children(u)
    let rem = dp2(u, me, back) - w[u]
    if rem == 0:
        walk.push(u)
        return
    let tot = sum(w[v] for v in ch)
    fn emit_rest(skip: List<V>):
        for v in ch if v not in skip: walk.push(v)
    if not back:
        for v in ch if dp2(v, false, false) == rem:
            walk.push(u)
            build2(v, false, false)
            return
        if let (val, v1, v2) = opt2(ch, v => dp2(v, false, true) - w[v], v => dp2(v, true, false) - w[v]):
            if tot + val == rem:
                walk.push(u)
                build2(v1, false, true)
                emit_rest([v1, v2])
                build2(v2, true, false)
                return
        if not me:
            if let (val, v1, v2, v3) = opt3(
                ch,
                v => dp2(v, true, true) - w[v],
                v => dp2(v, false, true) - w[v],
                v => dp2(v, true, false) - w[v],
            ):
                if tot + val == rem:
                    build2(v1, true, true)
                    walk.push(u)
                    build2(v2, false, true)
                    emit_rest([v1, v2, v3])
                    build2(v3, true, false)
                    return
            if let (val, v1, v2) = opt2(ch, v => dp2(v, true, true) - w[v], v => dp2(v, false, false) - w[v]):
                if tot + val == rem:
                    emit_rest([v1, v2])
                    build2(v1, true, true)
                    walk.push(u)
                    build2(v2, false, false)
                    return
            for v in ch if tot - w[v] + dp2(v, false, false) == rem:
                emit_rest([v])
                walk.push(u)
                build2(v, false, false)
                return
    elif me:
        for v in ch if tot - w[v] + dp2(v, false, true) == rem:
            walk.push(u)
            build2(v, false, true)
            emit_rest([v])
            return
    else:
        for v in ch if tot - w[v] + dp2(v, true, true) == rem:
            emit_rest([v])
            build2(v, true, true)
            walk.push(u)
            return

// k = 3: alternating pre-order / post-order DFS visits every vertex with steps <= 3.
fn build3(u: V, pre: Bool):
    if pre: walk.push(u)
    for v in tree.children(u): build3(v, not pre)
    if not pre: walk.push(u)

let ans = match k:
    1 =>
        build1(V.first)
        dp1(V.first)
    2 =>
        build2(V.first, true, false)
        dp2(V.first, true, false)
    3 =>
        build3(V.first, true)
        sum(w[v] for v in V)

print(ans)
print(walk.len)
print(" ".join("{v}" for v in walk))
