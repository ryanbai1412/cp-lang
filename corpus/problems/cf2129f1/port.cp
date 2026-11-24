// Source: Codeforces 2129F1 (Top-K Tracker, easy), https://codeforces.com/contest/2129/problem/F1
// Problem: interactive, multitest (t <= 40): find a hidden permutation p of n <= 845. A query
//   names distinct indices and gets back the largest min(60, m) of their p values (type 1) or pos
//   values (type 2); types 3 and 4 return min(300, m). At most 30 queries, one of type 3/4.
// Solution: give each value its own subset of the 30 queries: the empty set, 30 singletons, all
//   435 pairs and 379 triples placed greedily on the queries with the most room, so query 0 (type
//   4) holds at most 300 values and each other (type 2) at most 60 (2037 of 2040 slots). A query
//   asks for the positions of its values; holding no more than it returns, it returns them all, so
//   each position collects its value's subset, which a map inverts. 30 queries, O(30 n) work.
// Bounds: n: 2..=845

@!interactive
domain Query = 0..30
domain MaxVal = 1..=845

let room_of: [Query]Int = [Query] q => if q == Query.first: 300 else: 60

fn next_triple(avail: List<Query>, chosen: Set<Int>) -> ?Int:
    for (i, a) in avail.items:
        for (j, b) in avail.items if j > i:
            for c in avail[j + 1..]:
                let m = (1 << a.ord) | (1 << b.ord) | (1 << c.ord)
                if not chosen.contains(m): return Some(m)
    None

// Masks stay as `Int` bit codes until `Bitset<Query>` is hashable in `Set`/`Map`
// (`cpl::CpHash` rejects `cpl::Row<u64, 1>` in g++; see friction).
fn build_masks() -> [MaxVal]Int:
    var room = room_of
    var chosen = Set<Int>()
    var out: List<Int> = []
    fn take(m: Int):
        chosen.insert(m)
        out.push(m)
        for q in Query if m.bit(q.ord): room[q] -= 1
    take(0)
    for a in Query: take(1 << a.ord)
    for (a, b) in Query.pairs: take((1 << a.ord) | (1 << b.ord))
    while out.len < MaxVal.count:
        var avail = [q for q in Query if room[q] > 0]
        avail.sort(by: q => desc(room[q]))
        take(next_triple(avail, chosen)!)
    [MaxVal] v => out[v.ord]

let masks = build_masks()
var value_of = Map<Int, MaxVal>()
for v in MaxVal: value_of[masks[v]] = v

@!multitest
let n: 2..=845 = read()
domain Pos = 1..=n
var sig: [Pos]Int = 0
for q in Query:
    let held = [v for v in MaxVal if v.ord < n and masks[v].bit(q.ord)]
    if held.is_empty: continue
    let kind = if q == Query.first: 4 else: 2
    print(kind, held.len, held)
    let resp: [0..held.len]Pos = read()
    for p in resp: sig[p] |= 1 << q.ord
print("!", [value_of[sig[p]] for p in Pos])
