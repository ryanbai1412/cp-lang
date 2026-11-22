// SAC '22 Code Challenge 1 P5: n piles in a row, pile i holding w[i] items at height x[i].
// A prefix of the piles may be flattened to height 0, and the rest is cut into exactly k
// consecutive groups, each levelled to one height of its choice. Moving one item by one
// unit of height costs 1. Minimise the total cost.
// (Statement reconstructed from the original, corpus/submissions/3992625.cpp; the bound on
// n is a guess.)
// DRAFT 1: written for k = 1 (one group after the flattened prefix), a 1D scan over the cut.
let n: 1..=300 = read()
let k: 1..=n = read()
domain Pile = 0..n
let w: [Pile]1..=1e6 = read()
let x: [Pile]1..=1e6 = read()

// level[i, j]: cheapest way to level piles i..=j to one height, which is a weighted median
var level: [Pile, Pile]Int = 0
for i in Pile:
    for j in i..:
        let group = sorted([(x[p], w[p]) for p in i..=j])
        let half = sum(c for (_, c) in group) / 2
        var below = 0
        for (h, c) in group:
            below += c
            if below > half:
                level[i, j] = sum(c2 * abs(h - h2) for (h2, c2) in group)
                break

var answer: ?Int = none
var flattened = 0
for i in Pile:
    let b = level[i, Pile.last]
    answer = min(answer ?? b, flattened + b)
    flattened += x[i] * w[i]
print(answer!)
