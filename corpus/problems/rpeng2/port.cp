// Source: RPENG 2, which is IOI 1998 Polygon, https://dmoj.ca/problem/ioi98p6
// limits: 1000 ms, 32 MB
// Problem: a polygon of N <= 50 integer vertices whose edges are + ('t') or * ('x'). Remove one
//   edge, then repeatedly merge two neighbours by their edge's operation. Print the best final
//   value, then every edge (increasing) whose removal can reach it.
// Solution: removing edge i leaves the chain of vertices i, i + 1, ... round the ring, and a
//   chain's value comes from its last merge. Products of negatives need both ends, so span[i,
//   len] keeps the least and largest value of the chain of len + 1 vertices from i: + adds lows
//   and highs, * takes the extremes of the four products. O(N^3).
// original: corpus/submissions/4810287.cpp
// Bounds: n: 3..=50; ring.x: -32768..=32767
let n: 3..=50 = read()
domain V = 0..n
domain Len = 0..n
type Span = (lo: Int, hi: Int)
// Edge v joins vertex v - 1 and vertex v.
let ring: [V](op: Char, x: -32768..=32767) = read()

fn join(op: Char, a: Span, b: Span) -> Span:
    if op == 't': return (lo: a.lo + b.lo, hi: a.hi + b.hi)
    let products = [a.lo * b.lo, a.lo * b.hi, a.hi * b.lo, a.hi * b.hi]
    (lo: products.min()!, hi: products.max()!)

var span: [V, Len]Span = (lo: 0, hi: 0)
for i in V: span[i, Len.first!] = (lo: ring[i].x, hi: ring[i].x)
for len in Len.drop(1):
    for i in V:
        let parts = [
            join(
                ring[j].op,
                span[i, Len(left)],
                span[j, Len(len.ord - left - 1)],
            ) for left in 0..len.ord for j in [V.nth((i.ord + left + 1) % n)]
        ]
        span[i, len] = (lo: parts.map(p => p.lo).min()!, hi: parts.map(p => p.hi).max()!)

let best = V.map(i => span[i, Len.last!].hi).max()!
print(best)
print([i.ord + 1 for i in V if span[i, Len.last!].hi == best])
