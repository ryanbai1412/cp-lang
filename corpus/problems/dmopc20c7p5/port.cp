// DMOPC '20 C7 P5: `n` items with costs `cost[i]`, and `m` pairs `(u, v, w)`
// paying reward `w` when both items `u` and `v` are bought. Maximize total
// reward minus total cost.
//
// Project selection: each item has profit `-cost[i]`, each pair has profit `w`
// and depends on both its items `u` and `v`, so the maximum net gain is the
// maximum-weight closure of the dependency graph (`std.flow.max_closure`).

use std.flow.{max_closure}

let n: 1..=5_000 = read()
let m: 0..=250_000 = read()
domain Item = 1..=n
domain Pair = 0..m
let cost: [Item]Int = read()
let pairs: [Pair](u: Item, v: Item, w: Int) = read()

// Flatten `Item + Pair` into `Node` until sum domains land; `max_closure` owns src/snk.
domain Node = 0..n + m
fn item_node(i: Item) -> Node = Node(i.ord)
fn pair_node(p: Pair) -> Node = Node(n + p.ord)

var profit: [Node]Int = 0
for i in Item: profit[item_node(i)] = -cost[i]
for p in Pair: profit[pair_node(p)] = pairs[p].w

// taking a pair forces both of its items
var before: [Node]List<Node> = []
for p in Pair:
    before[item_node(pairs[p].u)].push(pair_node(p))
    before[item_node(pairs[p].v)].push(pair_node(p))

print(max_closure(before, profit: v => profit[v]).value)
