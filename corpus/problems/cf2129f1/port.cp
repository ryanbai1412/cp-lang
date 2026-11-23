// Codeforces 2129 F1: Top-K Everywhere (easy version, interactive).
// Assign each value v in 1..=845 a distinct subset `masks[v]` of 30 queries.
// Query 0 (type 4) returns up to 300 positions; queries 1..30 (type 2) each
// return up to 60 positions. As long as each query q holds at most `room_of[q]`
// values, it returns all of their positions, so every position p collects its
// exact value's mask `sig[p]` and `value_of[sig[p]]` recovers the permutation.
//
// Room check: all 1 + 30 + C(30, 2) = 466 masks of size <= 2 plus 379 triples
// total 845 masks using 30 + 2 * 435 + 3 * 379 = 2037 bit-slots, within the
// 300 + 29 * 60 = 2040 total capacity. Taking each triple on the three queries
// with the most remaining room fills all 845 without exhausting any query.

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
