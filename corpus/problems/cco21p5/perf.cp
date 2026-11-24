// Tuned: Ints with a `far` sentinel in the tree instead of `?Int`, one sorted list of packed
//   (found from, town) keys instead of a list per town, and a pointer for the next cut instead
//   of a binary search.
//
// Source: CCO 2021 P5, Bread First Search, https://dmoj.ca/problem/cco21p5
// limits: 1000 ms, 1024 MB
// Problem: an undirected simple graph with n <= 2e5 towns and m <= 2e5 roads. Print the least
//   number of new roads that make 1, 2, ..., n a valid BFS order (from town 1).
// Solution: BFS layers are consecutive blocks; town v (not 1) needs its earliest neighbour
//   `earliest[v]` in the previous block, or one new road. best[c] = least roads for the towns
//   from cut c on when a block starts at c. For a block starting at town i, the next block may
//   start only at a cut from which no town has an earliest neighbour before i (it would have
//   been found sooner): `later` is the suffix min of `earliest`, so the first such cut is a
//   binary search. Scanning i down, a town whose earliest neighbour is i, or who has none,
//   costs one road for every next-block start past it: a lazy SegTree of ?Int minimums with
//   range add (cuts not yet solved are None). O((n + m) log n).
// original: corpus/submissions/4554642.cpp
// Bounds: n: 2..=2e5; m: 1..=2e5; a, b: 1..=n
let (n, m): (2..=2e5, 1..=2e5) = read()
domain Town = 1..=n
var earliest: [Town]?Town = None
for (a, b) in read<[m](Town, Town)>():
    let (lo, hi) = minmax(a, b)
    earliest[hi] min= Some(lo)
// each town found from an earlier one, as e << 18 | v, sorted by the town e it is found from
var found: List<Int> = []
for v in Town:
    if let e = earliest[v]: found.push(e.ord << 18 | v.ord)
found.sort()
var unseen = found.len
var later: [Town.cuts]?Town = None
for v in Town.rev: later[v.l] = min(later[v.r], earliest[v])

// no way yet is `far`, beyond any count of towns
let far = 1 << 40
var best = SegTree(min, [Town.cuts] c => if c == Town.cuts.last!: 0 else: far, act: add)
let end = Town.cuts.last!
var next = end
for i in Town.drop(1).rev:
    while unseen > 0 and found[unseen - 1] >> 18 == i.ord:
        unseen -= 1
        best.apply(Town.nth(found[unseen] % (1 << 18)).r..=end, 1)
    if earliest[i] == None: best.apply(i.r..=end, 1)
    // the first cut after i with no town found before i beyond it only moves left as i does
    while next > i.r and later[Town.cuts(Int(next) - 1)].is_none_or(e => e >= i): next = Town.cuts(Int(next) - 1)
    best.set(i.l, best.fold(next..=end))
print(best.get(Town(2).l))
