@!multitest
let (n, m, q): (1..=300_000, 1..=300_000, 1..=300_000) = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V, w: 0..=300_000) = read()
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

var conn = Dsu<V>()
var ans_dsu = Dsu<V>()
var rep_hist = List<V>()
var ans = 0
var num_links = 0

fn add_range(l: Int, r: Int, cost: Int):
    for w in l..=r:
        for e in by_w[w]:
            if let (kept, merged) = conn.merge(e.u, e.v):
                if let y = rep[merged]:
                    if let x = rep[kept]:
                        if ans_dsu.unite(x, y):
                            ans += cost
                            num_links += 1
                    else:
                        rep_hist.push(kept)
                        rep[kept] = y

fn undo_to(snap: Int, hist_snap: Int):
    conn.undo_to(snap)
    while rep_hist.len > hist_snap:
        rep[rep_hist.pop()!] = none

fn solve(lo: Int, hi: Int):
    if num_links == num_good - 1 or lo == hi: return
    let mid = (lo + hi) / 2

    let (snap1, h1) = (conn.mark(), rep_hist.len)
    add_range(mid + 1, hi, lo)
    solve(lo, mid)
    undo_to(snap1, h1)

    if num_links == num_good - 1: return
    let (snap2, h2) = (conn.mark(), rep_hist.len)
    add_range(lo, mid, mid + 1)
    solve(mid + 1, hi)
    undo_to(snap2, h2)

solve(0, m)
print(if num_links == num_good - 1: ans else: -1)
