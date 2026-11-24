// Source: NOI 2016 Interval, https://dmoj.ca/problem/noi16p4
// Problem: n <= 5e5 closed intervals [l, r], 0 <= l <= r <= 1e9. Choose m <= 2e5 of them sharing
//   a common point, minimizing (longest length - shortest length), length = r - l. Print the
//   minimum, or -1 if no m intervals share a point.
// Solution: sorted by length, a best choice may as well be a window. Two pointers: for each left
//   end, extend the window until some point is covered m times, kept by a max/add lazy SegTree
//   over the compressed endpoints; take the length gap, then drop the left interval.
//   O(n log n).
// original: corpus/submissions/4791678.cpp
// Bounds: n: 1..=5e5; m: 1..=2e5; given.l, given.r: 0..=1e9

let (n, m): (1..=5e5, 1..=2e5) = read()
domain I = 0..n
let given: [I](l: 0..=1e9, r: 0..=1e9) = read()
fn length(s: (l: Int, r: Int)) -> Int = s.r - s.l
let spans = given.sorted(by: length)

let (X, coord) = compress([e for s in spans for e in [s.l, s.r]])
var depth = SegTree<X, Int, Int>(max, fill: 0, act: add)
fn cover(s: (l: Int, r: Int), times: Int):
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
