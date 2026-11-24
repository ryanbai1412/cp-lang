// Source: DMOPC '21 Contest 8 P1 (Peanut Planning), https://dmoj.ca/problem/dmopc21c8p1
// limits: 2000 ms, 256 MB
// Problem: n <= 5e5 numbers a_i <= 1e9 and m <= 1e9. Arrange them in a line so that every two
//   neighbours sum to at least m; print the lexicographically smallest such line, or -1.
// Solution: greedy: the line alternates the smallest unused number x and the smallest unused
//   number y with x + y >= m (y also fits the next, larger, x). As x grows, more of the largest
//   numbers become good partners and stay good, so in the sorted array small numbers leave
//   from the front and partners are promoted from the back onto a stack whose top is the
//   smallest. No partner left means -1. O(n log n) for the sort, then O(n).
// original: corpus/submissions/4522590.cpp
let n: 1..=1e6 = read()
let m: 1..=2e9 = read()
let a: [n]1..=1e9 = read()

// The line takes the smallest number left, then the smallest that is a good
// neighbour for it, and so on. The smallest numbers only grow, so a number
// that was once a good neighbour stays one: `small` gives numbers out from its
// front and promotes them from its back, and the promoted ones form a stack
// with the smallest on top.
var small = a.sorted()
var front = 0
var partners: List<Int> = []
var line: List<Int> = []
var stuck = false
while line.len < n and not stuck:
    var x = 0
    if front < small.len:
        x = small[front]
        front += 1
    else: x = partners.pop()!
    line.push(x)
    while front < small.len and small.last! + x >= m: partners.push(small.pop()!)
    if line.len < n:
        if let y = partners.pop(): line.push(y)
        else: stuck = true
if stuck: print(-1)
else: print(line)
