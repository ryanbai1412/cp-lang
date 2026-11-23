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
// The 29 type-2 queries recover each position's mask, which identifies either
// a single value or a pair `(v0, v1)` sharing a <= 2-bit mask. One type-3
// query (returning up to `TYPE3_LIMIT = 300` values) on one random position
// per pair resolves every pair in a single round.
use std.rand.{Rng}

@!interactive
const TYPE3_LIMIT = 300
domain Q = 0..29
domain Val = 1..=890

fn bits(qs: List<Int>) -> Int = qs.map(q => 1 << q).sum()
let rows = [bits([3 * i, 3 * i + 1, 3 * i + 2]) for i in 0..9]
let diags = [bits([3 * i, (3 * i + 4) % 27, (3 * i + 8) % 27]) for i in 0..9]
let small = [0] + [bits([q.ord]) for q in Q] + [bits([q.ord, r.ord]) for (q, r) in Q.pairs]
let all_masks = rows + diags + small + small
let masks = [Val] v => all_masks[v.ord]

var holders = Map<Int, List<Val>>()
for v in Val: holders[masks[v]] = (holders.get(masks[v]) ?? []) + [v]
var rng = Rng.new(50)

@!multitest
let n: 2..=890 = read()
domain Pos = 1..=n
fn holding(q: Q) -> List<Val> = [v for v in Val if v.ord < n and masks[v].bit(q.ord)]
fn ask_positions(vals: List<Val>) -> List<Pos>:
    print(2, vals.len, vals)
    let found: [0..vals.len]Pos = read()
    [p for p in found]

var sig: [Pos]Int = 0
for q in Q:
    let vals = holding(q)
    if not vals.is_empty:
        for p in ask_positions(vals): sig[p] |= 1 << q.ord

var at = Map<Int, List<Pos>>()
for p in Pos: at[sig[p]] = (at.get(sig[p]) ?? []) + [p]

var ans: [Pos]Val = Val.first!
var undo: [Val]?(Pos, Pos) = None
var probes = List<Pos>()
for (s, ps) in at:
    let vs = holders[s]
    if ps.len == 1:
        ans[ps[0]] = vs[0]
    else:
        let (p0, p1) = if rng.int(0, 1) == 1: (ps[1], ps[0]) else: (ps[0], ps[1])
        (ans[p0], ans[p1]) = (vs[0], vs[1])
        probes.push(p0)
        undo[vs[1]] = Some((p0, p1))

if not probes.is_empty:
    print(3, probes.len, probes)
    let reported: [0..min(probes.len, TYPE3_LIMIT)]Val = read()
    for v in reported if let (p0, p1) = undo[v]: ans.swap(p0, p1)
print("!", ans)
