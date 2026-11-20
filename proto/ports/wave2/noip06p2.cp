// NOIP 2006 Jinming's Budget: buy items within a budget to maximise the sum
// of price * importance; an accessory may only be bought with its main item.
// Each main item becomes a group of bundles (itself plus any set of its
// accessories), and the groups go through a 0/1 knapsack.
let (budget, n): (1..=32000, 1..=60) = read()
domain Item = 1..=n
let items: [Item](price: 1..=10000, weight: 1..=5, main: 0..=n) = read()

var extras: [Item]List<Item> = []
for i in Item if items[i].main != 0: extras[Item(items[i].main)].push(i)

var best: [0..=budget]Int = 0
for i in Item if items[i].main == 0:
    let it = items[i]
    var bundles = [(cost: it.price, worth: it.price * it.weight)]
    for e in extras[i]:
        let x = items[e]
        bundles += [(b.cost + x.price, b.worth + x.price * x.weight) for b in bundles]
    for j in (0..=budget).rev:
        for b in bundles if b.cost <= j: best[j] max= best[j - b.cost] + b.worth
print(best[budget])
