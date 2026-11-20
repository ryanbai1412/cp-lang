// IATI 2021 A1 P2, matching supply to demand up a weighted tree
// original: corpus/submissions/5071843.cpp
let n: 1..=2e5 = read()
domain V = 1..=n
let want: [V]Int = read()
let give: [V]Int = read()
let up: [V.first.r..](p: V, w: Int) = read()          // parents come first: p < v

// supply still unclaimed in a subtree, as (value at the subtree's root, count);
// `lift` is added to every stored value, so climbing an edge is O(1)
struct Pile:
    lift: Int = 0
    lots: MaxHeap<(Int, Int)> = MaxHeap()
    fn push(mut self, value: Int, count: Int): lots.push((value - lift, count))
    fn pop(mut self) -> ?(Int, Int):
        let (v, c) = lots.pop()?
        (v + lift, c)

var pile: [V]Pile = Pile()
var total = 0
for u in V.rev:
    // serve u from the most valuable supply below, never at a loss
    var used = 0
    while used < want[u]:
        let (value, count) = pile[u].pop() else: break
        if value < 0:
            pile[u].push(value, count)
            break
        let take = min(count, want[u] - used)
        total += value * take
        used += take
        if take < count: pile[u].push(value, count - take)
    // served demand may be re-routed later, so it re-enters as supply at value 0
    pile[u].push(0, used)
    pile[u].push(0, give[u])

    // hand the pile to the parent, small into large
    if u == V.first: break
    let (p, w) = up[u]
    pile[u].lift += w
    if pile[p].lots.len < pile[u].lots.len: swap(mut pile[p], mut pile[u])
    while let (value, count) = pile[u].pop(): pile[p].push(value, count)
print(total)
