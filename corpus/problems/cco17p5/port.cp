// Source: CCO 2017 P5 (Professional Network), https://dmoj.ca/problem/cco17p5
// Problem: n <= 2e5 people; person i befriends Kevin once he has A_i <= n friends, or at any time
//   for B_i <= 1e4 points. Print the fewest points to befriend all n.
// Solution: go through k = n down to 0, keeping a min-heap of the unpaid people with A >= k. They
//   cannot help each other reach k friends, so the n - (heap size) others must number at least k;
//   while they do not, pay the cheapest in the heap. O(n log n).
// original: corpus/submissions/4575278.cpp
// Bounds: n: 1..=2e5; people.need: 0..=n; people.price: 0..=1e4

let n: 1..=2e5 = read()
let people: [n](need: 0..=n, price: 0..=1e4) = read()

var by_need: [0..=n]List<0..=1e4> = []
for p in people: by_need[p.need].push(p.price)

var waiting = MinHeap<Int>()
var paid = 0
for k in (0..=n).rev:
    for price in by_need[k]: waiting.push(price)
    while n - waiting.len < k: paid += waiting.pop()!
print(paid)
