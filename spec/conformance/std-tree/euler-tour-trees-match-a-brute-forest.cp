// expect: ok, output "YES"
use std.tree.{EulerTourTree, LazyEulerTourTree}

domain V = 0..9

// The vertices `s` reaches, without the edge `block`.
fn brute_side(adj: [V]List<V>, s: V, block: ?(V, V)) -> List<V>:
    var seen: [V]Bool = false
    var q = [s]
    seen[s] = true
    var i = 0
    while i < q.len:
        let u = q[i]
        i += 1
        for w in adj[u]:
            if block == Some((u, w)) or block == Some((w, u)): continue
            if not seen[w]:
                seen[w] = true
                q.push(w)
    q

fn drop_one(mut l: List<V>, x: V):
    var k = 0
    while l[k] != x: k += 1
    l[k] = l.last!
    let _ = l.pop()

for _ in 0..30:
    var w: [V]Int = [V] _ => rand(-5, 5)
    var et = EulerTourTree(+, a: Some(w))
    let add = Action<Int, Int>(
        0,
        (f, g) => f + g,
        (d, x, n) => x + d * n,
        Monoid<Int>(0, (a, b) => a + b),
    )
    var lz = LazyEulerTourTree<V, Int, Int>.new(add, Some(w))
    var adj: [V]List<V> = [V] _ => []
    var edges: List<(V, V)> = []
    for _ in 0..200:
        let op = rand(0, 5)
        let u = V.nth(rand(0, V.count - 1))
        let v = V.nth(rand(0, V.count - 1))
        let cu = brute_side(adj, u, None)
        if op == 0 and not cu.contains(v):
            et.link(u, v)
            lz.link(u, v)
            adj[u].push(v)
            adj[v].push(u)
            edges.push((u, v))
        elif op == 1 and not edges.is_empty:
            let k = rand(0, edges.len - 1)
            let (a, b) = edges[k]
            edges[k] = edges.last!
            let _ = edges.pop()
            if rand(0, 1) == 0: et.cut(a, b) else: et.cut(b, a)
            lz.cut(a, b)
            drop_one(mut adj[a], b)
            drop_one(mut adj[b], a)
        elif op == 2:
            let x = rand(-5, 5)
            et.set(u, x)
            lz.set(u, x)
            w[u] = x
        elif op == 3 and not edges.is_empty:
            let (a, b) = edges[rand(0, edges.len - 1)]
            let side = brute_side(adj, a, Some((a, b)))
            let d = rand(-3, 3)
            assert et.fold_sub(a, b) == sum([w[y] for y in side])
            lz.apply_sub(a, b, d)
            for y in side:
                w[y] += d
            et = EulerTourTree(+, a: Some(w))
            for (p, q) in edges: et.link(p, q)
            assert lz.fold_sub(a, b) == sum([w[y] for y in side])
        elif op == 4:
            let d = rand(-3, 3)
            lz.apply_tree(u, d)
            for y in cu:
                w[y] += d
                et.set(y, w[y])
        assert et.connected(u, v) == brute_side(adj, u, None).contains(v)
        assert lz.connected(u, v) == et.connected(u, v)
        let cu2 = brute_side(adj, u, None)
        assert et.len(u) == cu2.len and lz.len(u) == cu2.len
        assert et.fold_tree(u) == sum([w[y] for y in cu2])
        assert lz.fold_tree(u) == sum([w[y] for y in cu2])
        assert et.get(v) == w[v] and lz.get(v) == w[v]
print("YES")
