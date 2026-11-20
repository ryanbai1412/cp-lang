// NOI '20 P2 - Destiny (DMOJ: noi20p2, 25pt, TL 2.0s, ML 1024MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Tree on n <= 5e5 vertices rooted at 1, m <= 5e5 ancestor-descendant path
// constraints (u_i, v_i) (u_i strict ancestor of v_i). Count 0/1 edge labelings
// where every path (u_i, v_i) contains at least one 1, mod 998244353.
//
// Tree DP f[u][d] (0 <= d < dep[u]) = ways in subtree u where deepest uncovered
// constraint starting inside subtree u has top depth d (0 = all covered).
// Maintained by Implicit Lazy Segment Tree Merging over `Pool<Node>` + `Node.Id`.
use std.graph.{Graph}
use std.tree.{Rooted}
use std.mod.{Mod}

type M = Mod<998244353>

let n: 1..=5e5 = read()
domain V = 1..=n
domain E = 0..(n - 1)
// WISH A9: Tree edge + strict-ancestor constraint `(u, v)` is non-typeable for
// `cpl stress`: `u` must be a strict ancestor of `v` in the read tree!
let edges: [E](u: V, v: V) = read()
let m: 0..=5e5 = read()
domain C = 0..m
let cons: [C](u: V, v: V) = read()

struct Node:
    l: ?Node.Id = none
    r: ?Node.Id = none
    sum: M = M(0)
    mul: M = M(1)

fn solve() -> M:
    let g = Graph<V, E>.undirected(edges)
    let tree = Rooted<V>(u => g.adj(u), V(1))

    // Depth 1-based so d = 0 means "all constraints satisfied".
    var dep: [V]Int = [V] v => tree.depth[v] + 1
    var max_top: [V]Int = [V] _ => 0
    for c in C:
        max_top[cons[c].v] max= dep[cons[c].u]

    // WISH A10: `Pool<Node>` + `Node.Id` implicit tree ergonomics:
    // (1) Every node access writes `pool[id].field` with no `pool.alloc(Node(...))`
    //     shorthand or scoped handle sugar, and (2) `merge(p, q)` mutates two
    //     distinct nodes in `pool`, so passing `mut pool` while reading `pool[p]`
    //     and `pool[q]` collides with `law disjoint` unless you copy fields out
    //     into locals before recursing!
    var pool = Pool<Node>.with_capacity(n * 22)

    mut fn apply_mul(id: ?Node.Id, k: M):
        if let u = id:
            pool[u].sum = pool[u].sum * k
            pool[u].mul = pool[u].mul * k

    mut fn push(u: Node.Id):
        let k = pool[u].mul
        if k != M(1):
            apply_mul(pool[u].l, k)
            apply_mul(pool[u].r, k)
            pool[u].mul = M(1)

    mut fn pull(u: Node.Id):
        let ls = if let lc = pool[u].l: pool[lc].sum else: M(0)
        let rs = if let rc = pool[u].r: pool[rc].sum else: M(0)
        pool[u].sum = ls + rs

    mut fn make_unit(lo: Int, hi: Int, pos: Int) -> Node.Id:
        let u = pool.alloc(Node(sum: M(1)))
        if lo < hi:
            let mid = (lo + hi) / 2
            if pos <= mid:
                let lc = make_unit(lo, mid, pos)
                pool[u].l = lc
            else:
                let rc = make_unit(mid + 1, hi, pos)
                pool[u].r = rc
        u

    mut fn range_sum(id: ?Node.Id, lo: Int, hi: Int, ql: Int, qr: Int) -> M:
        if id == none or qr < lo or hi < ql: return M(0)
        let u = id!
        if ql <= lo and hi <= qr: return pool[u].sum
        push(u)
        let mid = (lo + hi) / 2
        range_sum(pool[u].l, lo, mid, ql, qr) + range_sum(pool[u].r, mid + 1, hi, ql, qr)

    mut fn merge(p: ?Node.Id, q: ?Node.Id, lo: Int, hi: Int, pre_u: M, pre_c: M) -> ?Node.Id:
        if p == none and q == none: return none
        if q == none:
            apply_mul(p, pre_c)
            return p
        if p == none:
            apply_mul(q, pre_u)
            return q
        let (pu, qu) = (p!, q!)
        if lo == hi:
            let vu = pool[pu].sum
            let vc = pool[qu].sum
            pool[pu].sum = vu * (pre_c + vc) + vc * pre_u
            return pu
        push(pu)
        push(qu)
        let (pl, pr) = (pool[pu].l, pool[pu].r)
        let (ql, qr) = (pool[qu].l, pool[qu].r)
        let lu = if let x = pl: pool[x].sum else: M(0)
        let lc = if let x = ql: pool[x].sum else: M(0)
        let mid = (lo + hi) / 2
        pool[pu].l = merge(pl, ql, lo, mid, pre_u, pre_c)
        pool[pu].r = merge(pr, qr, mid + 1, hi, pre_u + lu, pre_c + lc)
        pull(pu)
        pu

    // Bottom-up over reverse preorder avoids 5e5 recursion depth stack overflow.
    var roots: [V]?Node.Id = [V] _ => none
    // WISH A11: `Rooted<V>` still lacks `.postorder()` / `.order.rev` iteration
    // without dropping to raw `.pre` indices.
    for u in tree.postorder():
        roots[u] = make_unit(0, n, max_top[u])
        for c in tree.children(u):
            let s_c = range_sum(roots[c], 0, n, 0, dep[u])
            roots[u] = merge(roots[u], roots[c], 0, n, M(0), s_c)

    range_sum(roots[V(1)], 0, n, 0, 0)

print(solve().val)
