// DMOPC '20 C7 P5 (dream): items with costs, pairs (u, v, w) paying w when both
// u and v are bought. Maximize reward minus cost. Project selection: a pair is
// profit w, an item is profit -cost, and buying a pair needs both its items, so
// the answer is the maximum closure of the needs graph.
// Needs sum domains: `Item + Pair` is one index space whose elements are
// either an item or a pair, so no hand offsets like `n + e`.
with io(base: 1)

domain Item = read()
domain Pair = read()
let cost: [Item]Int = read()
let pairs: [Pair](u: Item, v: Item, w: Int) = read()

domain Node = Item + Pair
let profit = [Node] x => match x:
    Item(i) => -cost[i]
    Pair(p) => pairs[p].w
var unlocks: [Item]List<Node> = []
for p in Pair:
    for i in [pairs[p].u, pairs[p].v]:
        unlocks[i].push(Node(p))
fn before(x: Node) -> List<Node> = match x:
    Item(i) => unlocks[i]
    Pair(p) => []
print(max_closure(before: before, profit: x => profit[x]).value)

// The same cut as an explicit network, for models that are not a closure:
//
//     domain Net = Item + Pair + labels(S, T)
//     var net = FlowGraph<Net>()
//     for i in Item: net.add(Net(i), Net.T, cost[i])
//     for p in Pair:
//         net.add(Net.S, Net(p), pairs[p].w)
//         net.add(Net(p), Net(pairs[p].u), cap: none)
//         net.add(Net(p), Net(pairs[p].v), cap: none)
//     print(sum(pairs[p].w for p in Pair) - net.max_flow(Net.S, Net.T))
