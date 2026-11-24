// Source: ETSR 2023 Round 1 P2, https://dmoj.ca/problem/etsr23r1p2
// Problem: q <= 2e5 orders on a set of ranges in 1..=n (n <= 1e9), starting empty. Kind
//   2 adds l..=r after throwing out every range that touches it and prints how many it
//   threw out; kind 1 prints how many ranges there are.
// Solution: ranges in the set never overlap, so they sort the same by start and by end.
//   A max SegTree over compressed coordinates marks the starts: throw out every start
//   in l..=r, then the last range starting before l if it reaches l. Each range is
//   thrown out once, O(q log q).
// original: corpus/submissions/5494248.cpp
// Bounds: q: 1..=2e5; n: 1..=1e9
let q: 1..=2e5 = read()
let n: 1..=1e9 = read()
type Order = (kind: 1..=2, l: 1..=1e9, r: 1..=1e9)
fn read_order() -> Order:
    let kind: 1..=2 = read()
    if kind == 1: return (1, 1, 1)
    let l: 1..=n = read()
    let r: l..=n = read()
    (2, l, r)
var orders: List<Order> = []
for _ in 0..q: orders.push(read_order())

// No two ranges of the set overlap, so they are in the same order by start and
// by end. The starts live in a max-tree over the compressed coordinates, which
// stands in for an ordered set: a cell holds its own number while a range
// starts there and 0 otherwise.
let ends = [o.l for o in orders] + [o.r for o in orders]
let (X, coord) = compress(ends)
var starts = SegTree<X, Int>(max, fill: 0)
var end_of: [X]Int = 0  // where the range that starts here ends
var held = 0
fn throw_out(x: X) -> Int:
    starts.set(x, 0)
    held -= 1
    1
for o in orders:
    if o.kind == 1:
        print(held)
        continue
    let first = coord.first_ge(o.l)!
    let last = coord.last_le(o.r)!
    var gone = 0
    while starts.fold(first..=last) > 0: gone += throw_out(X(starts.fold(first..=last) - 1))
    if let before = X.at(first - 1):
        let top = starts.fold(X.first!..=before)  // the last range that starts before l
        if top > 0 and end_of[X(top - 1)] >= o.l: gone += throw_out(X(top - 1))
    starts.set(first, first.idx + 1)
    end_of[first] = o.r
    held += 1
    print(gone)
