// Runnable today/ port of EGOI '22 P7 (Toy Design) for proto/cpc.
// Embeds the local persistent-DSU grader from stdin (n, m, then m hidden edges) and runs
// the D&C segment tree solver with generation-stamped Marks<Pin> (O(1) clear).
// Prints the query count and the reconstructed spanning forest edges.
// original: corpus/submissions/5203797.cpp

fn main_solve():
    let n: Int = read()
    let m: Int = read()
    domain Pin = 0..n

    // Local grader: persistent list of DSU parent arrays
    var g_par: List<List<Int>> = []
    var init_p: List<Int> = [-1 for i in 0..n]
    g_par.push(init_p)

    fn g_find(ver: Int, x: Int) -> Int:
        var u = x
        while g_par[ver][u] >= 0: u = g_par[ver][u]
        return u

    fn g_unite(ver: Int, a: Int, b: Int):
        var u = g_find(ver, a)
        var v = g_find(ver, b)
        if u == v: return
        if g_par[ver][u] < g_par[ver][v]:
            swap(mut u, mut v)
        g_par[ver][v] += g_par[ver][u]
        g_par[ver][u] = v

    for _ in 0..m:
        let u: Int = read()
        let v: Int = read()
        g_unite(0, u - 1, v - 1)

    var queries = 0
    fn query_conn(a: Int, u: Pin, v: Pin) -> Int:
        queries += 1
        if g_find(a, u.idx) == g_find(a, v.idx): return a
        var copy: List<Int> = [x for x in g_par[a]]
        let nv = g_par.len
        g_par.push(copy)
        g_unite(nv, u.idx, v.idx)
        return nv

    // Solver state: DSU + D&C segment tree + generation-stamped Marks<Pin>
    var dsu = Dsu<Pin>()
    var ver = [0..4 * n + 5] _ => 0
    var rep = [0..4 * n + 5] _ => Pin(0)
    var mark_l: [Pin]Int = 0
    var mark_r: [Pin]Int = 0
    var gen = 0

    fn find_match(o: Int, l: Int, r: Int, x: Pin) -> Pin:
        if l == r: return Pin(l)
        let mid = (l + r) / 2
        let lc = o * 2
        let rc = lc + 1
        var has_l = false
        for i in l..=mid:
            if mark_r[dsu.find(Pin(i))] != gen:
                has_l = true
                break
        if not has_l: return find_match(rc, mid + 1, r, x)
        var has_r = false
        for i in mid + 1..=r:
            if mark_r[dsu.find(Pin(i))] != gen:
                has_r = true
                break
        if not has_r: return find_match(lc, l, mid, x)
        if query_conn(ver[lc], x, rep[lc]) == ver[lc]:
            return find_match(lc, l, mid, x)
        return find_match(rc, mid + 1, r, x)

    fn build(o: Int, l: Int, r: Int):
        if l == r:
            ver[o] = 0
            rep[o] = Pin(l)
            return
        let mid = (l + r) / 2
        let lc = o * 2
        let rc = lc + 1
        build(lc, l, mid)
        build(rc, mid + 1, r)
        gen += 1
        ver[o] = ver[rc]
        for i in l..=mid:
            let p = Pin(i)
            let rp = dsu.find(p)
            if mark_l[rp] == gen: continue
            mark_l[rp] = gen
            let cv = query_conn(ver[o], p, rep[rc])
            if cv != ver[o]:
                ver[o] = cv
                continue
            let rv = find_match(rc, mid + 1, r, p)
            dsu.unite(p, rv)
            let nr = dsu.find(p)
            mark_l[nr] = gen
            mark_r[nr] = gen
        rep[o] = rep[lc]

    build(1, 0, n - 1)
    var edges: List<(Int, Int)> = []
    for i in Pin:
        let p = dsu.find(i)
        if p != i: edges.push((p.idx + 1, i.idx + 1))
    print(queries)
    print(edges.len)
    for e in edges:
        print(e.0, e.1)

main_solve()
