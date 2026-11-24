// Source: DMOPC '20 Contest 7 P5 (Mayors and Tolls), https://dmoj.ca/problem/dmopc20c7p5
// Problem: N <= 500 cities with mayor fees b_i and M <= 500 roads (u, v, p): a road earns p if
//   both its cities' mayors are paid (values <= 1e9). Print the best total profit minus fees.
// Solution: project selection: each city has profit -b_i, each road profit p and requires both
//   its cities, so the answer is the maximum-weight closure of that dependency graph
//   (`std.flow.max_closure`, a min cut) over N + M nodes.

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
