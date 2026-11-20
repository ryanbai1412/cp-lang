// Runnable port (proto/cp.py lib-run) of dmopc21c2p5.cp
// original: corpus/submissions/3968384.cpp
use std.graph.{Graph}
use std.tree.{Rooted}
use std.mod.{Mod}

const MOD = 998_244_352
const MAX_DEG = 26
type Z = Mod<MOD>

let (n, k): (Int, Int) = read()
domain V = 1..=n
domain E = 0..(n - 1)
domain Deg = 0..MAX_DEG

let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.undirected(edges)
let tree = Rooted<V>.new(u => g.adj(u), V.first)

var fact: [Deg]Z = [Deg] _ => Z(1)
for d in Deg:
    if d.to_idx > 0:
        fact[d] = fact[Deg(d.to_idx - 1)] * Z(d.to_idx)

var memo0: [V]List<Z> = [V] _ => []
var memo1: [V]List<Z> = [V] _ => []

// Walk vertices in reverse preorder (children before parent).
var pre_order: List<V> = []
for p in tree.sub(V.first):
    pre_order.push(tree.at[p])

for u in pre_order.rev:
    let sz_u = tree.size[u]

    for par_used in 0..=1:
        var cur: [Deg]List<Z> = [Deg] _ => [Z(0) for _ in 0..=sz_u]
        cur[Deg(par_used)][par_used] = Z(1)
        var csum = par_used

        for v in tree.kids[u]:
            let sz_v = tree.size[v]
            let c0 = memo0[v]
            let c1 = memo1[v]
            var nxt: [Deg]List<Z> = [Deg] _ => [Z(0) for _ in 0..=sz_u]
            for d in Deg:
                let di = d.to_idx
                for i in 0..=csum:
                    let stay = cur[d][i]
                    let prev_d = if di > 0: cur[Deg(di - 1)][i] else: Z(0)
                    if stay != Z(0) or prev_d != Z(0):
                        let jmax = min(sz_v, sz_u - i)
                        for j in 0..=jmax:
                            nxt[d][i + j] = nxt[d][i + j] + stay * c0[j]
                            if j > 0:
                                nxt[d][i + j] = nxt[d][i + j] + prev_d * c1[j]
            csum += sz_v
            cur = nxt

        var res: List<Z> = []
        for i in 0..=sz_u:
            var acc = Z(0)
            for d in Deg:
                acc = acc + cur[d][i] * fact[d]
            res.push(acc)
        if par_used == 0:
            memo0[u] = res
        else:
            memo1[u] = res

if k <= n:
    print(memo0[V.first][k].v)
else:
    print(0)
