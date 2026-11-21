// Codeforces 2129 F2 (dream, interactive). Give each value 1..=890 a mask over
// 29 query bits, each bit used at most 60 times: 18 triples on bits 0..27, any
// two sharing at most one bit, then two copies of every set of at most two bits
// (values 19..=454 and 455..=890). The 29 type-2 queries recover each position's
// mask, which names one value, or a pair (v0, v1) with v0 <= 454 < v1 when both
// copies of a small set are present. One type-3 probe on a random position of
// each pair resolves every pair at once.
@!interactive
use std.rand.{Rng}

domain Q = 0..29
domain Val = 1..=890
type Mask = Bitset<Q>

let triples = [Mask([3 * i, 3 * i + 1, 3 * i + 2]) for i in 0..9]
            + [Mask([3 * i, (3 * i + 4) % 27, (3 * i + 8) % 27]) for i in 0..9]
let small = [Mask([])] + [Mask([q]) for q in Q] + [Mask([q, r]) for (q, r) in Q.pairs]
let masks: [Val]Mask = triples + small + small
let holders = Val.group(by: v => masks[v])
var rng = Rng(50)

@!multitest
let n: 2..=890 = read()
domain Pos = 1..=n
var sig: [Pos]Mask = Mask([])
for q in Q:
    let vals = Val.filter(v => v <= n and q in masks[v])
    if vals.is_empty: continue
    for p in ask<[vals.len]Pos>(2, vals.len, vals): sig[p].add(q)

var ans: [Pos]Int = 0
var undo: [Val]?(Pos, Pos) = none
var probes: List<Pos> = []
for (s, ps) in Pos.group(by: p => sig[p]):
    let vs = holders[s]
    if ps.len == 1:
        ans[ps[0]] = vs[0]
    else:
        let (p0, p1) = if rng.coin(): (ps[1], ps[0]) else: (ps[0], ps[1])
        (ans[p0], ans[p1]) = (vs[0], vs[1])
        probes.push(p0)
        undo[vs[1]] = (p0, p1)

if not probes.is_empty:
    for v in ask<[min(probes.len, 300)]Val>(3, probes.len, probes):
        if let (p0, p1) = undo[v]: ans.swap(p0, p1)
print("!", ans)
