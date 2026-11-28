// Source: Codeforces 2129F2 (Top-K Tracker (Hard Version)),
//   https://codeforces.com/contest/2129/problem/F2
// limits: 3000 ms, 250 MB
// Problem: interactive, multitest (t <= 40). Find a hidden permutation p of n <= 890 with at most
//   30 queries on index sets: types 1/2 return the top min(60, m) of the p (resp. pos) values,
//   types 3/4 the top min(300, m), and at most one query of type 3 or 4.
// Solution: give each value a mask over the 29 type-2 queries so that every query asks about at
//   most 60 values. Bits 0..26 form a 9 x 3 grid; its 9 rows and 9 wrapped diagonals are 18
//   triples, any two sharing at most one bit. Then two copies of every mask of at most 2 bits
//   (436 each: values 19..=454 and 455..=890). A bit is in 2 triples and 2 * (1 + 28) small
//   masks: 60. The type-2 answers spell each position's mask, which names one value or a pair
//   (v, v + 436). One type-3 query on a random position of each pair (at most 436) reports every
//   larger value among them, since about half hold one, far below 300. O(n^2).
// Bounds: n: 2..=890; found: 1..=890
use std.rand.{Rng}

let wire = Wire()
const TYPE3_LIMIT = 300
domain Q = 0..29
domain Val = 1..=890

fn mask(qs: List<Q>) -> Int = qs.map(q => 1 << q.ord).sum()
fn cell(row: Int, col: Int) -> Q = Q(3 * (row % 9) + col)
fn grid_line(start: Int, shift: Int) -> Int = mask([cell(start + shift * col, col) for col in 0..3])
let triples = [grid_line(i, shift) for shift in 0..2 for i in 0..9]
let small = [0] + [mask([q]) for q in Q] + [mask([q, r]) for (q, r) in Q.pairs]
let all_masks = triples + small + small
let masks = [Val] v => all_masks[v.ord]
let holders = Val.group(by: v => masks[v])
var rng = Rng.new(50)

@!multitest
let n: 2..=890 = read()
domain Pos = 1..=n
fn positions_holding(q: Q) -> List<Pos>:
    let vals = [v for v in Val if Int(v) <= n and masks[v].bit(q.ord)]
    if vals.is_empty: return []
    print(2, vals.len, vals)
    let found: [0..vals.len]Pos = read()
    found.collect()
fn top_values_at(probes: List<Pos>) -> List<Val>:
    if probes.is_empty: return []
    print(3, probes.len, probes)
    let found: [0..min(probes.len, TYPE3_LIMIT)]Val = read()
    found.collect()

var sig: [Pos]Int = 0
for q in Q:
    for p in positions_holding(q): sig[p] |= 1 << q.ord

let order: [Pos]Pos = rng.perm()
let at = order.group(by: p => sig[p])
let reported = Set(top_values_at([ps[0] for ps in at.values if ps.len == 2]))
fn value_at(p: Pos) -> Val:
    let (ps, vs) = (at[sig[p]], holders[sig[p]])
    if ps.len == 1: return vs[0]
    let larger_at = if vs[1] in reported: ps[0] else: ps[1]
    if p == larger_at: vs[1] else: vs[0]
print("!", [Pos] p => value_at(p))
