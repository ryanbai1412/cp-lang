// Source: CCO 2021 P5, Bread First Search, https://dmoj.ca/problem/cco21p5
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
let (n, m): (2..=2e5, 1..=2e5) = read()
domain Town = 1..=n
var earliest: [Town]?Town = None
for (a, b) in read<[m](Town, Town)>():
    let (lo, hi) = minmax(a, b)
    earliest[hi] min= Some(lo)
var found_by: [Town]List<Town> = []
for v in Town:
    if let e = earliest[v]: found_by[e].push(v)
var later: [Town.cuts]?Town = None
for v in Town.rev: later[v.l] = min(later[v.r], earliest[v])

let add_min = Action<?Int, Int>(0, (f, g) => f + g, (f, x, _) => x.map(y => y + f), over: Monoid<?Int>(None, (x, y) => min(x, y)))
var best = SegTree(add_min.over, [Town.cuts] c => if c == Town.cuts.last!: Some(0) else: None, act: add_min)
let end = Town.cuts.last!
for i in Town.drop(1).rev:
    for v in found_by[i]: best.apply(v.r..=end, 1)
    if earliest[i] == None: best.apply(i.r..=end, 1)
    let next = Town.cuts((Int(i.r)..=n).first_true(k => later[Town.cuts(k)].is_none_or(e => e >= i))!)
    best.set(i.l, best.fold(next..=end))
print(best.get(Town(2).l)!)
