// NOIP 2010 Tortoise Chess: a track of n squares, each with a score, and m
// cards, each moving the tortoise 1 to 4 squares; the cards add up to
// exactly n - 1. Play them in any order; the score is the sum over every
// square stood on, the first included. Print the best score.
// The cards left decide the square, so the state is four counts.
// original: corpus/submissions/5445662.cpp
let n: 1..=350 = read()
let m: 1..=120 = read()
let score: [n]0..=100 = read()
let cards: [m]1..=4 = read() where cards.sum() == n - 1
let have = [count(c == step for c in cards) for step in 1..=4] where have.max()! <= 40
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
