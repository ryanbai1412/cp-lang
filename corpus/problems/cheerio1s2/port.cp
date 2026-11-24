// Source: Cheerio Contest 1 S2 (School Expansion), https://dmoj.ca/problem/cheerio1s2
// limits: 1000 ms, 512 MB
// Problem: build N <= 1e5 classrooms on floors with no gaps; the j-th room on floor i costs
//   A i + B j (A, B <= 1e4). Print the least total cost (needs 64 bits).
// Solution: cost grows along floors and along rooms, so the N cheapest (floor, room) pairs never
//   leave a gap. A MinHeap holds each floor's next room (A i + B for floors 1..=N); the N
//   cheapest pops, each pushing that floor's next room (+B), are the answer. O(N log N).
// original: corpus/submissions/5337643.cpp
// Bounds: n: 1..=1e5; a, b: 1..=1e4
let n: 1..=1e5 = read()
let a: 1..=1e4 = read()
let b: 1..=1e4 = read()

var next = MinHeap<Int>()
for i in 1..=n: next.push(a * i + b)
var total = 0
for _ in 0..n:
    let w = next.pop()!
    total += w
    next.push(w + b)
print(total)
