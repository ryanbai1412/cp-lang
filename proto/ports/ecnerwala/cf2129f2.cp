// Codeforces 2129 F2: Permutation (Hard Version), interactive.
//
// Give each value 1..=890 a mask over 29 type-2 queries (capacity 60 each):
// - View bits 0..27 as a 9 x 3 grid `3 * row + col`. The 9 rows
//   `{3i, 3i + 1, 3i + 2}` and 9 shifted diagonals `{3i, 3i + 4, 3i + 8} mod 27`
//   pick one bit per column with distinct row shifts, so any two triples
//   share at most one bit and each bit in 0..27 is in two of them.
// - Two copies of every set of size <= 2 on 0..29 (1 + 29 + 406 = 436 sets).
//   Each bit is used at most 2 + 2 * (1 + 28) = 60 times.
// The 29 type-2 queries recover each position's mask, which names either one
// value or a pair (v0, v1) sharing a small mask. One type-3 query on one random
// position per pair (answered with up to 300 values) resolves every pair: if
// it reports v1, the guessed order was wrong.
use std.rand.{Rng}

@!interactive
domain Val = 1..=890
fn bits(qs: List<Int>) -> Int = qs.map(q => 1 << q).sum()
let triples = [
    m for i in 0..9 for m in [
        bits([3 * i, 3 * i + 1, 3 * i + 2]),
        bits([3 * i, (3 * i + 4) % 27, (3 * i + 8) % 27]),
    ]
]
let small = [0] + [bits([q]) for q in 0..29] + [bits([q, r]) for q in 0..29 for r in q + 1..29]
let all_masks = triples + small + small
let masks = [Val] v => all_masks[v.ord]

var holders = Map<Int, List<Val>>()
for v in Val: holders[masks[v]] = (holders.get(masks[v]) ?? []) + [v]
var rng = Rng.new(50)

let t: 1..=40 = read()
for _ in 0..t:
    let n: 2..=890 = read()
    domain Pos = 1..=n
    fn holding(q: Int) -> List<Int> = [v for v in 1..=n if masks[Val(v)].bit(q)]
    fn ask_positions(vals: List<Int>) -> List<Pos>:
        print(2, vals.len, vals)
        let found: [0..vals.len]Pos = read()
        [p for p in found]

    var sig: [Pos]Int = 0
    for q in 0..29:
        let vals = holding(q)
        if not vals.is_empty:
            for p in ask_positions(vals): sig[p] |= 1 << q

    var at = Map<Int, List<Pos>>()
    for p in Pos: at[sig[p]] = (at.get(sig[p]) ?? []) + [p]

    var ans: [Pos]Int = 0
    var undo: [Val]?(Pos, Pos) = none
    var probes = List<Pos>()
    for (s, ps) in at:
        let vs = holders[s]
        if ps.len == 1:
            ans[ps[0]] = vs[0].ord + 1
        else:
            let (p0, p1) = if rng.int(0, 1) == 1: (ps[1], ps[0]) else: (ps[0], ps[1])
            (ans[p0], ans[p1]) = (vs[0].ord + 1, vs[1].ord + 1)
            probes.push(p0)
            undo[vs[1]] = some((p0, p1))

    if not probes.is_empty:
        print(3, probes.len, probes)
        let reported: [0..min(probes.len, 300)]Val = read()
        for v in reported if let (p0, p1) = undo[v]: ans.swap(p0, p1)
    print("!", ans)
