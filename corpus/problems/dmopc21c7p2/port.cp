// Source: DMOPC '21 Contest 7 P2, Knitting Scarves, https://dmoj.ca/problem/dmopc21c7p2
// limits: 2000 ms, 256 MB
// Problem: the scarf starts as colours 1..N (N <= 1e6). Q <= 5e5 steps each cut the segment from
//   colour L to colour R and paste it right after colour K (K = 0: at the front; K is outside the
//   segment). Print the final order.
// Solution: keep the scarf as a doubly linked list over spots 0..=N, spot 0 being the front
//   and `next` optional past the last colour; each step splices L..R out and back in after K
//   by relinking three pairs, skipped when it already follows K. Walk the list to print.
//   O(N + Q).
// original: corpus/submissions/4452915.cpp
// Node 0 is the head sentinel; `top + 1` is the first node no splice can move.
// Bounds: n: 1..=1e6; q: 1..=5e5; ops.l, ops.r, ops.k: 0..=n

let n: 1..=1e6 = read()
let q: 1..=5e5 = read()
domain Spot = 0..=n  // 0 is the front of the scarf; spot c >= 1 is colour c
let ops: [q](l: Spot, r: Spot, k: Spot) = read()

var next: [Spot]?Spot = [Spot] s => s.next
var prev: [Spot]Spot = [Spot] s => s.prev ?? s

fn link(a: Spot, b: ?Spot):
    next[a] = b
    if let b = b: prev[b] = a

for (l, r, k) in ops if next[k] != Some(l):
    let after_k = next[k]
    let before_l = prev[l]
    let after_r = next[r]
    link(k, l)
    link(r, after_k)
    link(before_l, after_r)

var scarf: List<Spot> = []
var at = next[Spot.first!]
while let s = at:
    scarf.push(s)
    at = next[s]
print(scarf)
