// Assign each value 1..=890 a 29-bit mask (each bit used <= 60 times):
// 18 unique triples plus two copies of all 436 subsets of size <= 2.
// 29 type-2 queries recover every position's mask, narrowing each to a
// unique value or a colliding pair (v0, v1) with v0 <= 454 < v1. One
// randomized type-3 probe on one position per colliding pair resolves all pairs.
@!interactive
use std.rand.{Rng}

let masks = do:
    var ms: List<Int> = []
    for i in 0..9:
        ms.push((1 << (3 * i)) | (1 << (3 * i + 1)) | (1 << (3 * i + 2)))
        ms.push((1 << (3 * i)) | (1 << ((3 * i + 4) % 27)) | (1 << ((3 * i + 8) % 27)))
    for _ in 0..2:
        ms.push(0)
        for i in 0..29:
            ms.push(1 << i)
            for j in i + 1..29: ms.push((1 << i) | (1 << j))
    ms
let mask_vals = (1..=890).group(by: v => masks[v - 1])
var rng = Rng(50)

--- multitest ---
let n: 2..=890 = read()
domain Pos = 1..=n
var sig: [Pos]Int = 0

for q in 0..29:
    let vals = [v for v in 1..=n where masks[v - 1].bit(q)]
    if vals.is_empty: continue
    for p in ask<[vals.len]Pos>(2, vals.len, vals): sig[p] |= 1 << q

var ans: [Pos]Int = 0
var swap_pair: [1..=n]?(Pos, Pos) = none
var probes: List<Pos> = []

for (s, ps) in Pos.group(by: p => sig[p]):
    let vs = mask_vals[s]
    if ps.len == 1:
        ans[ps[0]] = vs[0]
    else:
        var (p0, p1) = (ps[0], ps[1])
        if rng.int(0, 2) == 1: swap(mut p0, mut p1)
        (ans[p0], ans[p1]) = (vs[0], vs[1])
        probes.push(p0)
        swap_pair[vs[1]] = (p0, p1)

if not probes.is_empty:
    for v in ask<[min(probes.len, 300)](1..=n)>(3, probes.len, probes):
        if let (p0, p1) = swap_pair[v]: ans.swap(p0, p1)

print("!", ans)
