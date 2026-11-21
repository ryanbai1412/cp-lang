// Give each value its own set of queries (a mask over 30 queries) and put the
// value in every query of its set. Query 0 is the single type-4 query, which
// returns up to 300 positions; queries 1..30 are type-2, 60 each. A query that
// holds no more values than it returns gives back the positions of all of them,
// so each position collects exactly its value's mask. 845 masks fit the room:
// every mask of size at most 2, then masks of size 3, each on the three queries
// with the most room left.
@interactive
type Query = 0..30
let room_of: [Query]Int = [q => if q == 0: 300 else: 60]

let masks: List<Set<Query>> = do:
    var room = room_of
    var chosen = SortedSet<Set<Query>>()
    fn take(s: Set<Query>):
        chosen.add(s)
        for q in s: room[q] -= 1
    for s in Query.subsets(size: ..=2): take(s)
    while chosen.len < 845:
        let roomiest = Query.sort(by: q => desc(room[q]))
        take(roomiest.subsets(size: 3).first(s => s not in chosen and s.all(q => room[q] > 0))!)
    chosen.to_list()
let value_of = masks.inverse()

@!multitest
let n: 2..=845 = read()
domain Val = 1..=n
domain Pos = 1..=n
var mask: [Pos]Set<Query> = {}
for q in Query:
    let held = [v for v in Val if q in masks[v - 1]]
    if held.is_empty: continue
    for p in ask<[held.len]Pos>(if q == 0: 4 else: 2, held.len, held): mask[p].add(q)
print("!", [value_of[mask[p]]! + 1 for p in Pos])
