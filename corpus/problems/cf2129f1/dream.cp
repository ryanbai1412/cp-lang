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

let wire = Wire()
domain Query = 0..30
domain MaxVal = 1..=845
type Mask = Set<Query>
let no_queries: List<Query> = []

let room_of: [Query]Int = [Query] q => if q == Query.first: 300 else: 60

fn build_masks() -> [MaxVal]Mask:
    var room = room_of.clone()
    let no_masks: List<Mask> = []
    var chosen = Set(no_masks)
    var out: List<Mask> = []
    fn take(s: Mask):
        chosen.add(s)
        out.push(s)
        for q in s: room[q] -= 1
    // The first unused triple with room, scanning the roomiest queries first.
    fn next_triple(order: List<Query>) -> ?Mask:
        for i in order.keys:
            for j in i + 1..order.len:
                for k in j + 1..order.len:
                    let s: Mask = Set([order[i], order[j], order[k]])
                    if s not in chosen and [q for q in s].all(q => room[q] > 0): return Some(s)
        None
    take(Set(no_queries))
    for a in Query: take(Set([a]))
    for (a, b) in Query.pairs: take(Set([a, b]))
    while out.len < MaxVal.count:
        take(next_triple(Query.sort(by: q => desc(room[q])))!)
    [MaxVal] v => out[v.ord]

let masks = build_masks()
var value_of: Map<Mask, MaxVal> = Map()
for (v, m) in masks.items: value_of[m] = v

fn query_positions(q: Query, held: List<MaxVal>) -> List<Int>:
    let kind = if q == Query.first: 4 else: 2
    print(kind, held.len, held)
    read<[0..held.len]Int>().collect()

@!multitest
let n: 2..=845 = read()
domain Pos = 1..=n
var mask: [Pos]Mask = [Pos] _ => Set(no_queries)
for q in Query:
    let held = [v for v in MaxVal if v.ord < n and q in masks[v]]
    if not held.is_empty:
        for p in query_positions(q, held): mask[Pos(p)].add(q)
print("!", [value_of.get(mask[p])! for p in Pos])
