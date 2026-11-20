// SAC '22 Code Challenge 1 P5: n piles in a row, pile i holding w[i] items at height x[i].
// A prefix of the piles may be flattened to height 0, and the rest is cut into exactly k
// consecutive groups, each levelled to one height of its choice. Moving one item by one
// unit of height costs 1. Minimise the total cost.
// (Statement reconstructed from the original, corpus/submissions/3992625.cpp; the bound on
// n is a guess.)
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

// best[i, g]: piles from position i on, cut into exactly g groups (position n is the end)
var best: [0..=n, 0..=k]?Int = none
best[n, 0] = 0
for i in (0..n).rev:
    for g in 1..=k:
        best[i, g] = min(level[Pile(i), Pile(j - 1)] + rest for j in i + 1..=n if let rest = best[j, g - 1])

var answer: ?Int = none
var flattened = 0
for i in Pile:
    if let b = best[i.idx, k]: answer = min(answer ?? b, flattened + b)
    flattened += x[i] * w[i]
print(answer!)
