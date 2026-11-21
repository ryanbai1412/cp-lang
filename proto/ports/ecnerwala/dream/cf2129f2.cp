// Codeforces 2129 F2: Top-K Everywhere (hard version, interactive).
// Give each value 1..=890 a mask over 29 type-2 queries (capacity 60 each):
//   - View bits 0..27 as a 9 x 3 grid `(row, col) = 3 * row + col`. Take the
//     9 rows `{3i, 3i + 1, 3i + 2}` and the 9 shifted diagonals
//     `{3i, 3((i + 1) % 9) + 1, 3((i + 2) % 9) + 2}` (18 triples). Each triple
//     picks one bit per column with distinct row shifts 0, +1, +2 mod 9, so any
//     two triples share at most one bit and each bit in 0..27 appears in 2.
//   - Two copies of every set of size <= 2 on 0..29 (1 + 29 + 406 = 436 sets,
//     covering values 19..=454 and 455..=890). Each bit is used at most
//     2 + 2 * (1 + 28) = 60 times.
// The 29 type-2 queries (`query_positions`) recover each position's mask, which
// identifies either a single value or a pair `(v0, v1)` sharing a <= 2-bit mask.
// One type-3 query (`query_values`, returning up to `TYPE3_LIMIT = 300` values)
// on one random position per pair resolves every pair in a single round.

@!interactive
use std.rand.Rng

const TYPE3_LIMIT = 300
domain Q = 0..29
domain Val = 1..=890
type Mask = Bitset<Q>

let rows = [Mask([3 * i, 3 * i + 1, 3 * i + 2]) for i in 0..9]
let diags = [Mask([3 * i, (3 * i + 4) % 27, (3 * i + 8) % 27]) for i in 0..9]
let small = [Mask([])] + [Mask([q]) for q in Q] + [Mask([q, r]) for (q, r) in Q.pairs]
let masks: [Val]Mask = rows + diags + small + small
let holders = Val.group(by: v => masks[v])
var rng = Rng(50)

fn query_positions(vals: List<Val>) -> List<Int> = ask<[vals.len]Int>(2, vals.len, vals)
fn query_values(probes: List<Int>) -> List<Val> =
    ask<[min(probes.len, TYPE3_LIMIT)]Val>(3, probes.len, probes)

@!multitest
let n: 2..=890 = read()
domain Pos = 1..=n
var sig: [Pos]Mask = Mask([])
for q in Q:
    let vals = Val.filter(v => v.ord < n and q in masks[v])
    if not vals.is_empty:
        for p in query_positions(vals): sig[Pos(p)].add(q)

var ans: [Pos]Val = Val.first
var undo: [Val]?(Pos, Pos) = none
var probes: List<Int> = []
for (s, ps) in Pos.group(by: p => sig[p]):
    let vs = holders[s]
    if ps.len == 1:
        ans[ps[0]] = vs[0]
    else:
        let (p0, p1) = if rng.coin(): (ps[1], ps[0]) else: (ps[0], ps[1])
        (ans[p0], ans[p1]) = (vs[0], vs[1])
        probes.push(p0.ord + 1)
        undo[vs[1]] = (p0, p1)

if not probes.is_empty:
    for v in query_values(probes):
        if let (p0, p1) = undo[v]: ans.swap_at(p0, p1)
print("!", ans)
