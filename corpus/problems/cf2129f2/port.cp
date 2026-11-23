// Source: Codeforces 2129F2 (Top-K Tracker (Hard Version)),
//   https://codeforces.com/contest/2129/problem/F2
// Problem: interactive, multitest (t <= 40). Find a hidden permutation p of n <= 890 with at most
//   30 queries on index sets: types 1/2 return the top min(60, m) of the p (resp. pos) values,
//   types 3/4 the top min(300, m), and at most one query of type 3 or 4.
// Solution: give each value a set of the 29 type-2 queries (a bit mask) so that every query
//   holds at most 60 values: 18 triples meeting pairwise in at most one bit, then every mask of
//   at most 2 bits twice. The type-2 answers spell each position's mask, which names a value or a
//   pair of values (v, v + 436). Guess each pair at random and send one type-3 query with one
//   position of each pair; the larger values it returns show which guesses to swap. O(n^2).
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
    found.collect()

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
