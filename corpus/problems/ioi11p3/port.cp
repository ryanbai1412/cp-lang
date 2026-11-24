// Source: IOI 2011 Day 1 (Rice Hub), https://dmoj.ca/problem/ioi11p3
// limits: 600 ms, 256 MB
// Problem: n <= 1e5 rice fields at sorted integer coordinates in 1..=L (L <= 1e9, equal
//   coordinates allowed) and a budget B <= 2e15. Put a hub at an integer coordinate; bringing
//   a field's rice costs its distance. Print the most fields whose total cost is <= B.
// Solution: the fields taken are a contiguous block and the hub goes on its median field,
//   whose cost prefix sums give in O(1). The cost only grows as the block widens, so slide
//   two pointers: for each right end, advance the left end while over budget. O(n).
// original: corpus/problems/ioi11p3/orig.cpp (submission 3732163, its grader reading B as 64-bit)
let n: 1..=1e5 = read()
type Spot = 1..=1e9
let len: Spot = read()
let budget: 0..=2e15 = read()
domain Field = 1..=n

fn sorted_fields() -> [Field]Spot:
    var at: [Field]Spot = 1
    for i in Field: at[i] = rand(1, len)
    at.sort()
    at

let x: [Field]Spot = read(gen: sorted_fields)
let pre = prefix(x)

// the hub goes on the middle field of the block
fn cost(l: Field, r: Field) -> Int:
    let mid = Field((l.label + r.label) / 2)
    (pre.fold(mid..=r) - pre.fold(l..=mid)) - x[mid] * ((r - mid) - (mid - l))

var best = 0
var l = Field.first!
for r in Field:
    while cost(l, r) > budget: l = l.next!
    best max= r - l + 1
print(best)
