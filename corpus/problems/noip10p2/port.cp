// Source: NOIP 2010 Senior P2 (Tortoise Chess), https://dmoj.ca/problem/noip10p2
// limits: 1000 ms, 125 MB
// Problem: a row of N <= 350 squares scored 0..100, and M <= 120 cards each moving 1 to 4
//   squares, at most 40 of each kind, summing to exactly N - 1. Play all cards in some order
//   from square 1; print the best total score of the squares visited, the start included.
// Solution: the cards played fix the current square, so the state is the four counts played.
//   best(a, b, c, d) = this square's score + the max over each kind played of best with that
//   count lowered, filled in increasing order; at most 41^4 states, 4 moves each.
// original: corpus/submissions/5445662.cpp
// Bounds: n: 1..=350; m: 1..=120; score: 0..=100; cards: 1..=4
let n: 1..=350 = read()
let m: 1..=120 = read()
let score: [n]0..=100 = read()
let cards: [m]1..=4 = read()
let have = [count(c == step for c in cards) for step in 1..=4]
domain A = 0..=have[0]
domain B = 0..=have[1]
domain C = 0..=have[2]
domain D = 0..=have[3]

// best[a, b, c, d]: best score collected after playing a, b, c, d cards of each step
var best: [A, B, C, D]Int = 0
for a in A:
    for b in B:
        for c in C:
            for d in D:
                var before = 0
                if let a1 = a.prev: before max= best[a1, b, c, d]
                if let b1 = b.prev: before max= best[a, b1, c, d]
                if let c1 = c.prev: before max= best[a, b, c1, d]
                if let d1 = d.prev: before max= best[a, b, c, d1]
                best[a, b, c, d] = before + score[a.ord + 2 * b.ord + 3 * c.ord + 4 * d.ord]
print(best[A.last!, B.last!, C.last!, D.last!])
