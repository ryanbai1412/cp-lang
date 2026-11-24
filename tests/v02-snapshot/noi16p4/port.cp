// Source: NOI 2016 Interval, https://dmoj.ca/problem/noi16p4
// Problem: n <= 5e5 closed intervals [l, r], 0 <= l <= r <= 1e9. Choose m <= 2e5 of them sharing
//   a common point, minimizing (longest length - shortest length), length = r - l. Print the
//   minimum, or -1 if no m intervals share a point.
// Solution: sorted by length, a best choice may as well be a window. Two pointers: for each left
//   end, extend the window until some point is covered m times, kept by a max/add lazy SegTree
//   over the compressed endpoints; take the length gap, then drop the left interval.
//   O(n log n).
let n: 1..=500000 = read()
let m: 1..=200000 = read()
type Span = (l: 0..=1e9, r: 0..=1e9)
domain I = 0..n
fn short() -> Span:  // few coordinates, so intervals pile up
    let l = rand(0, 8)
    (l, l + rand(0, 5))
fn tangle() -> [I]Span: [I] _ => short()
let given: [I]Span = read(gen: tangle)
assert all(s.l <= s.r for s in given)
fn length(s: Span) -> Int: s.r - s.l
let spans = given.sorted(by: s => length(s))

var ends: List<Int> = []
for s in spans:
    ends.push(s.l)
    ends.push(s.r)
let (X, coord) = compress(ends)
var depth = SegTree<X, Int, Int>(max, fill: 0, act: add)
fn cover(s: Span, times: Int):
    depth.apply(coord.first_ge(s.l)!..=coord.first_ge(s.r)!, times)

var best: ?Int = None
var taken = 0  // the window is spans[i..taken]
for i in 0..n:
    while taken < n and depth.fold(X.all) < m:
        cover(spans[taken], 1)
        taken += 1
    if depth.fold(X.all) >= m:
        best min= length(spans[taken - 1]) - length(spans[i])
    cover(spans[i], -1)
print(best ?? -1)
