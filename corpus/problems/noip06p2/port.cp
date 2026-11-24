// Source: NOIP 2006 Senior P2 (Jinming's Budget Plan), Luogu P1064 (not on DMOJ)
// Problem: budget n <= 32000, m <= 60 items with price 0..=1e4, importance 1..5 and a main item
//   (0 if it is a main item; a main item has at most 2 accessories). An accessory can be bought
//   only with its main item. Print the largest sum of price * importance within the budget.
// Solution: each main item and its accessories form a group whose options are the main item plus
//   any subset of its accessories (at most 4); a grouped 0/1 knapsack over money, descending,
//   takes at most one option per group: O(n * m * 4).
// original: corpus/submissions/5165705.cpp
let budget: 1..=32000 = read()
let n: 1..=60 = read()
domain Item = 1..=n
domain Money = 0..=budget
let items: [Item](price: 0..=10000, weight: 1..=5, main: 0..=n) = read()

let owner = [Item] i => Item.at(items[i].main)  // None for a main item
fn worth(i: Item) -> Int = items[i].price * items[i].weight

var extras: [Item]List<Item> = []
for i in Item:
    if let o = owner[i]: extras[o].push(i)

var best: [Money]Int = 0
for i in Item:
    if owner[i] == None:
        var bundles: List<(cost: Int, worth: Int)> = [(items[i].price, worth(i))]
        for e in extras[i]:
            let plain = bundles.len  // bundles without this accessory
            for t in 0..plain:
                bundles.push((bundles[t].cost + items[e].price, bundles[t].worth + worth(e)))
        for j in Money.rev:
            for b in bundles:
                if let rest = Money.at(j - b.cost): best[j] max= best[rest] + b.worth
print(best[Money.last!])
