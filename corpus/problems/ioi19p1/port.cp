// Source: IOI 2019 Day 1 P1 (Arranging Shoes), https://dmoj.ca/problem/ioi19p1
// Problem: 2n shoes in a row (n <= 1e5), -s a left shoe of size s and +s a right one. With
//   swaps of adjacent shoes, make every pair of positions 2i, 2i+1 a left then a right shoe of
//   one size. Print the fewest swaps.
// Solution: fix the leftmost shoe: its partner should be the nearest unused shoe of the same
//   size and other foot (per size, a queue of shoes still waiting). Pull the partner next to
//   it, one swap per shoe still standing between them, plus one swap if the pair is the wrong
//   way round, then forget both. A SegTree of standing shoes counts them: O(n log n).
// original: corpus/submissions/5661847.cpp
let n: 1..=100000 = read()
domain Pos = 0..2 * n
let shoe: [Pos]Int = read()

domain Size = 1..=n

fn swaps(shoe: [Pos]Int) -> Int:
    // Per size, the shoes still waiting for a partner, oldest first. They
    // are all the same foot, so a new shoe either pairs with the oldest or
    // joins the queue.
    var waiting: [Size]List<Pos> = []
    var served: [Size]Int = 0
    var partner: [Pos]?Pos = None
    for p in Pos:
        let size = Size(abs(shoe[p]))
        let k = served[size]
        if k < waiting[size].len and shoe[waiting[size][k]] != shoe[p]:
            partner[waiting[size][k]] = p
            served[size] = k + 1
        else:
            waiting[size].push(p)
    var standing = SegTree<Pos, Int>(+, fill: 0)
    for p in Pos: standing.set(p, 1)
    var total = 0
    for p in Pos:
        if let q = partner[p]:
            standing.set(p, 0)
            standing.set(q, 0)
            total += standing.fold(p..=q)
            if shoe[p] > 0: total += 1
    total

print(swaps(shoe))
