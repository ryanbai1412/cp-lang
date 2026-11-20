// ETSR '23 Round 4 P1 — Permutation-Cycle Balanced Parentheses (CP Lang v2 spec)
// original: corpus/submissions/5521420.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Decompose permutation `p` via `Perm<V>.cycles()`. 2-cycles deterministically
// place '(' before ')'; longer even cycles are 2-colored over `Subset<Big>`
// until every prefix sum is non-negative.

let n: 2..=100 = read()
domain V = 1..=n
let p: Perm<V> = read()

var base: [V]Int = 0
var big: List<List<V>> = []
for cyc in p.cycles():
    if cyc.len % 2 != 0: return
    if cyc.len == 2:
        let (a, b) = (min(cyc[0], cyc[1]), max(cyc[0], cyc[1]))
        base[a] = 1
        base[b] = -1
    else:
        big.push(cyc)

let (Big, bcycles) = big.freeze()
for mask in Subset<Big>:
    var cur = base.copy()
    for c in Big:
        let flip = if c in mask: 1 else: 0
        for (k, v) in bcycles[c].items:
            cur[v] += ((k ^ flip) % 2) * 2 - 1
    if all(s >= 0 for s in scan(cur, +)):
        print(Str([if cur[v] == 1: '(' else: ')' for v in V]))
        return

@checker fn check():
    let out: Str = read_output()
    assert out.len == n
    var bal = 0
    for i in 0..n:
        bal += if out[i] == '(': 1 else: -1
        assert bal >= 0
    assert bal == 0
    var deg: [V]Int = 0
    for v in V if out[v.to_idx] == '(':
        deg[v] += 1
        deg[p[v]] += 1
    assert all(deg[v] == 1 for v in V)
