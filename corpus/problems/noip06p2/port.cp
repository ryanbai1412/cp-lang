// NOIP 2006 Jinming's Budget: buy items within a budget to maximise the sum
// of price * importance; an accessory may only be bought with its main item.
// Each main item becomes a group of bundles (itself plus any set of its
// accessories), and the groups go through a 0/1 knapsack.
let budget: 1..=32000 = read()
let n: 1..=60 = read()
domain Item = 1..=n
domain Money = 0..=budget
let items: [Item](price: 1..=10000, weight: 1..=5, main: 0..=60) = read()
assert all(it.main <= n for it in items)

fn worth(i: Item) -> Int: items[i].price * items[i].weight

var extras: [Item]List<Item> = []
for i in Item:
    if items[i].main != 0: extras[Item(items[i].main)].push(i)

var best: [Money]Int = 0
for i in Item:
    if items[i].main == 0:
        var bundles: List<(cost: Int, worth: Int)> = [(items[i].price, worth(i))]
        for e in extras[i]:
            let plain = bundles.len  // bundles without this accessory
            for t in 0..plain:
                bundles.push((bundles[t].cost + items[e].price, bundles[t].worth + worth(e)))
        for j in Money.rev:
            for b in bundles:
                if let rest = Money.at(j - b.cost): best[j] max= best[rest] + b.worth
print(best[Money.last!])
