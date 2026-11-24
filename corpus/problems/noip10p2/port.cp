// Source: NOIP 2010 Senior P2 (Tortoise Chess), https://dmoj.ca/problem/noip10p2
// limits: 1000 ms, 125 MB
// Problem: a row of N <= 350 squares scored 0..100, and M <= 120 cards each moving 1 to 4
//   squares, at most 40 of each kind, summing to exactly N - 1. Play all cards in some order
//   from square 1; print the best total score of the squares visited, the start included.
// Solution: the cards left fix the current square, so the state is the four counts left.
//   Memoised best(a, b, c, d) = this square's score + the max over each kind still left of
//   best with that count lowered; at most 41^4 states, 4 moves each.
// original: corpus/submissions/5445662.cpp
let n: 1..=350 = read()
let m: 1..=120 = read()
let score: [n]0..=100 = read()
let cards: [m]1..=4 = read()
let have = [count(c == step for c in cards) for step in 1..=4]
domain Left = 0..=40

// best score still to collect, standing on the square the played cards lead to
@memo
fn best(a: Left, b: Left, c: Left, d: Left) -> Int:
    let at = n - 1 - (a.ord + 2 * b.ord + 3 * c.ord + 4 * d.ord)
    var more = 0
    if let a1 = a.prev: more max= best(a1, b, c, d)
    if let b1 = b.prev: more max= best(a, b1, c, d)
    if let c1 = c.prev: more max= best(a, b, c1, d)
    if let d1 = d.prev: more max= best(a, b, c, d1)
    score[at] + more

print(best(Left(have[0]), Left(have[1]), Left(have[2]), Left(have[3])))
