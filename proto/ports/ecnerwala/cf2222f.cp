@!multitest
use std.dsu.{DsuOf}
let (n, m, q): (1..=300_000, 1..=300_000, 1..=300_000) = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V, w: 0..=m) = read()
let colors: [0..q]V = read()

var rep: [V]?V = none
var num_good = 0
for v in colors:
    if rep[v] == none:
        rep[v] = v
        num_good += 1

var by_w: [0..=m]List<(u: V, v: V)> = []
for e in edges:
    if e.u != e.v:
        by_w[e.w].push((u: e.u, v: e.v))

var conn = DsuOf<V, ?V>(v => rep[v], (a, b) => a ?? b)
var ans_dsu = Dsu<V>()
var ans = 0
var num_links = 0

fn add_range(l: Int, r: Int, cost: Int):
    for w in l..=r:
        for e in by_w[w]:
            let (x, y) = (conn.get(e.u), conn.get(e.v))
            if conn.unite(e.u, e.v):
                if let a = x:
                    if let b = y:
                        if ans_dsu.unite(a, b):
                            ans += cost
                            num_links += 1

fn solve(lo: Int, hi: Int):
    if num_links == num_good - 1 or lo == hi: return
    let mid = (lo + hi) / 2
    let snap = conn.mark()
    add_range(mid + 1, hi, lo)
    solve(lo, mid)
    conn.undo_to(snap)

    if num_links == num_good - 1: return
    add_range(lo, mid, mid + 1)
    solve(mid + 1, hi)
    conn.undo_to(snap)

solve(0, m)
print(if num_links == num_good - 1: ans else: -1)
