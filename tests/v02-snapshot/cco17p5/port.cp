// Source: CCO 2017 P5 (Professional Network), https://dmoj.ca/problem/cco17p5
// Problem: n <= 2e5 people; person i befriends Kevin once he has A_i <= n friends, or at any time
//   for B_i <= 1e4 points. Print the fewest points to befriend all n.
// Solution: go through k = n down to 0, keeping a min-heap of the unpaid people with A >= k. They
//   cannot help each other reach k friends, so the n - (heap size) others must number at least k;
//   while they do not, pay the cheapest in the heap. O(n log n).
// original: corpus/submissions/4575278.cpp
let n: 1..=2e5 = read()
let people: [n](need: 0..=n, price: 0..=1e4) = read()

var by_need: [0..=n]List<Int> = []
for p in people: by_need[p.need].push(p.price)

// From the most demanding down. `waiting` holds the people who want k friends
// or more and are not paid for. They cannot help each other reach k, so the
// other n - waiting.len people have to be k at least; while they are not,
// the cheapest of those waiting is paid.
var waiting = MinHeap<Int>()
var paid = 0
for k in (0..=n).rev:
    for price in by_need[k]: waiting.push(price)
    while n - waiting.len < k: paid += waiting.pop()!
print(paid)
