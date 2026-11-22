// expect: ok, output "YES"
use std.tree.{LinkCut, LazyLinkCut, cartesian_tree}
use std.rand.{Rng}
domain V = 0..8
fn path(adj: [V, V]Bool, a: V, b: V) -> ?List<V>:
    var back: [V]?V = None
    var seen: [V]Bool = false
    seen[a] = true
    var todo: List<V> = [a]
    var i = 0
    while i < todo.len:
        let u = todo[i]
        i += 1
        for w in V:
            if adj[u, w] and not seen[w]:
                seen[w] = true
                back[w] = Some(u)
                todo.push(w)
    if not seen[b]: return None
    var out: List<V> = [b]
    var x = b
    while x != a:
        x = back[x]!
        out.push(x)
    Some(out.rev.collect())
fn forests():
    var adj: [V, V]Bool = false
    var lc = LinkCut<V, Int>(+)
    var lz = LazyLinkCut<V, Int, Int>.new(
        Action<Int,
        Int>(0, (f, g) => f + g, (d, x, n) => x + d * n, Monoid<Int>(0, (a, b) => a + b)),
    )
    for step in 0..400:
        let (u, v) = (V.nth(rand(0, 7)), V.nth(rand(0, 7)))
        if step % 3 == 0 and u != v:
            if adj[u, v]:
                lc.cut(u, v)
                lz.cut(u, v)
                adj[u, v] = false
                adj[v, u] = false
            elif path(adj, u, v) == None:
                lc.link(u, v)
                lz.link(u, v)
                adj[u, v] = true
                adj[v, u] = true
        let p = path(adj, u, v)
        let want: ?Int = if let q = p: Some(q.len - 1) else: None
        assert lc.dist(u, v) == want and lz.dist(u, v) == want
        for k in 0..10:
            let at: ?V = if let q = p: (if k < q.len: Some(q[k]) else: None) else: None
            assert lc.jump(u, v, k) == at and lz.jump(u, v, k) == at
fn cartesian():
    domain D = 0..9
    for _ in 0..200:
        let a = [D] _ => rand(0, 4)
        let par = cartesian_tree(a)
        for i in D:
            // the parent is the larger of the nearest left `<=` and the nearest right `<`; a tie goes right
            var l: ?D = None
            for j in D:
                if j < i and a[j] <= a[i]: l = Some(j)
            var r: ?D = None
            for j in D.rev:
                if j > i and a[j] < a[i]: r = Some(j)
            let want: ?D = if l == None: r elif r == None: l elif a[l!] > a[r!]: l else: r
            assert par[i] == want
fn lazy_link_cut_matches_a_brute_forest():
    domain V = 1..=9
    var rng = Rng(11)
    var w = [V] _ => rng.int(-5, 5)
    let add = Action<Int, Int>(
        0,
        (f, g) => f + g,
        (d, x, n) => x + d * n,
        Monoid<Int>(0, (a, b) => a + b),
    )
    var lct = LazyLinkCut<V, Int, Int>.new(add, Some(w))
    var adj: [V]List<V> = []
    for _ in 0..400:
        let u = V(rng.int(1, 9))
        let v = V(rng.int(1, 9))
        var from: [V]?V = None
        var seen: [V]Bool = false
        seen[u] = true
        var st = [u]
        while let x = st.pop():
            for y in adj[x]:
                if not seen[y]:
                    seen[y] = true
                    from[y] = Some(x)
                    st.push(y)
        assert lct.connected(u, v) == seen[v]
        if seen[v]:
            var path = [v]
            var x = v
            while let p = from[x]:
                path.push(p)
                x = p
            let ab = path.rev.collect()
            assert lct.dist(u, v) == Some(ab.len - 1)
            let k = rng.int(0, ab.len)
            assert lct.jump(u, v, k) == (if k < ab.len: Some(ab[k]) else: None)
            let op = rng.int(0, 3)
            if op == 0:
                let d = rng.int(-3, 3)
                lct.apply_path(u, v, d)
                for y in ab: w[y] += d
            elif op == 1 and from[v] != None:
                let p = from[v]!
                lct.cut(v, p)
                adj[v] = [y for y in adj[v] if y != p]
                adj[p] = [y for y in adj[p] if y != v]
            elif op == 2:
                let s = rng.int(-5, 5)
                lct.set(v, s)
                w[v] = s
            assert lct.fold_path(u, v) == sum([w[y] for y in ab]) or op == 1
            assert lct.get(u) == w[u]
        else:
            assert lct.dist(u, v) == None and lct.jump(u, v, 0) == None
            lct.link(u, v)
            for (x, y) in [(u, v), (v, u)]: adj[x].push(y)

forests()
lazy_link_cut_matches_a_brute_forest()
cartesian()
print("YES")
