// NOI 2002 Greedy Nine-Headed Dragon: the fruits of a tree are shared among
// the heads. The big head eats exactly k fruits, the root among them, and
// every other head eats at least one. A branch is hard to swallow, at its
// own cost, when one head eats both its ends. Minimise the total cost.
// original: corpus/submissions/3718952.cpp
let n: 1..=300 = read()
let heads: 2..=300 = read()
let k: 1..=n = read()
domain Fruit = 1..=n
domain Branch = 1..n
type Twig = (a: Fruit, b: Fruit, cost: 0..=1e5)

fn random_tree() -> [Branch]Twig:
    [Branch] t => (a: Fruit(rand(1, t.label)), b: Fruit(t.label + 1), cost: rand(0, 100))
let branches: [Branch]Twig = read(gen: random_tree)

var adj: [Fruit]List<Fruit> = []
for (a, b, _) in branches:
    adj[a].push(b)
    adj[b].push(a)
let tree = Rooted(adj, Fruit.first!)
var up: [Fruit]Int = 0  // cost of the branch to the parent
for (a, b, cost) in branches:
    up[if tree.parent(a) == b: a else: b] = cost

// Only two things matter about a fruit: whether the big head eats it, and if
// not, whether the small heads can avoid meeting across a branch. With two or
// more small heads they always can, by alternating down the tree.
domain Who = 0..2  // 0: a small head, 1: the big head
let big = Who(1)
// cost[v, w, x]: cheapest sharing of v's subtree where w eats v and the big
// head eats x fruits of the subtree
var cost: [Fruit, Who, 0..=n]?Int = None
for v in Fruit:
    cost[v, Who(0), 0] = 0
    cost[v, big, 1] = 0
for c in tree.at.rev:  // children are complete before parents
    if let p = tree.parent(c):
        var merged: [Who, 0..=n]?Int = None
        for wp in Who:
            for wc in Who:
                let clash = wp == wc and (wp == big or heads == 2)
                let join = if clash: up[c] else: 0
                for x in 0..=n:
                    if let above = cost[p, wp, x]:
                        for y in 0..=n - x:
                            if let below = cost[c, wc, y]: merged[
                                wp,
                                x + y,
                            ] min= above + below + join
        for w in Who:
            for x in 0..=n: cost[p, w, x] = merged[w, x]
print(if heads - 1 + k > n: -1 else: cost[Fruit.first!, big, k]!)
