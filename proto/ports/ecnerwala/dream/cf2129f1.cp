// Codeforces 2129 F1: Top-K Everywhere (easy version, interactive).
// Assign each value v in 1..=845 a distinct subset `masks[v]` of 30 queries.
// Query 0 (type 4) returns up to 300 positions; queries 1..30 (type 2) each
// return up to 60 positions. As long as each query q holds at most `room_of[q]`
// values, it returns all of their positions, so every position p collects its
// exact value's mask `mask[p]` and `value_of[mask[p]]` recovers the permutation.
//
// Room check: all 1 + 30 + C(30, 2) = 466 masks of size <= 2 plus 379 triples
// total 845 masks using 30 + 2 * 435 + 3 * 379 = 2037 bit-slots, within the
// 300 + 29 * 60 = 2040 total capacity. Taking each triple on the three queries
// with the most remaining room fills all 845 without exhausting any query.

@!interactive
domain Query = 0..30
domain MaxVal = 1..=845
type Mask = Set<Query>

let room_of: [Query]Int = [Query] q => if q == Query.first: 300 else: 60

fn build_masks() -> [MaxVal]Mask:
    var room = room_of
    var chosen = SortedSet<Mask>()
    var out: List<Mask> = []
    fn take(s: Mask):
        chosen.add(s)
        out.push(s)
        for q in s: room[q] -= 1
    for s in Query.subsets(size: ..=2): take(s)
    while out.len < MaxVal.count:
        let roomiest = Query.sort(by: q => desc(room[q]))
        let next = roomiest.subsets(size: 3).find(s => s not in chosen and s.all(q => room[q] > 0))!
        take(next)
    [MaxVal] v => out[v.ord]

let masks = build_masks()
let value_of = masks.inverse()

fn query_positions(q: Query, held: List<MaxVal>) -> List<Int>:
    let kind = if q == Query.first: 4 else: 2
    ask<[held.len]Int>(kind, held.len, held)

@!multitest
let n: 2..=845 = read()
domain Pos = 1..=n
var mask: [Pos]Mask = {}
for q in Query:
    let held = [v for v in MaxVal if v.ord < n and q in masks[v]]
    if not held.is_empty:
        for p in query_positions(q, held): mask[Pos(p)].add(q)
print("!", [value_of[mask[p]]! for p in Pos])
