// NOI 2020 Day 2 P1, Dishes (CP Lang 0.2.2): greedy pairing via `SortedBag` +
// bitset subset-sum partition for the m = n - 2 case.
// original: corpus/submissions/3890337.cpp

type Stock = (grams: Int, id: Int)

fn cook(items: List<Stock>, k: Int):
    var bag = SortedBag<Stock>.of(items)
    while not bag.is_empty:
        let a = bag.pop_first()!
        if a.grams >= k:
            print(a.id, k)
            if a.grams > k: bag.add((grams: a.grams - k, id: a.id))
        else:
            let b = bag.pop_last()!
            let take = k - a.grams
            print(a.id, a.grams, b.id, take)
            if b.grams > take: bag.add((grams: b.grams - take, id: b.id))

@multitest
let (n, m, k): (Int, Int, Int) = read()
domain Item = 1..=n
let d: [Item]Int = read()
if m >= n - 1:
    cook([(grams: d[i], id: i.label) for i in Item], k)
    return

let lowest = sum(min(d[i] - k, 0) for i in Item)
let highest = sum(max(d[i] - k, 0) for i in Item)
domain Raise = 0..=(highest - lowest)
let goal = Raise.from_idx(-k - lowest)
var reach: List<Bitset<Raise>> = [Bitset<Raise>.new()]
reach[0].add(Raise(0))
for i in Item:
    var next = reach[i.idx].shifted(abs(d[i] - k))
    next.union(reach[i.idx])
    reach.push(next)
if goal == none or not reach[n].contains(goal!):
    print(-1)
    return

var at = goal!
var (group, others): (List<Stock>, List<Stock>) = ([], [])
for i in Item.rev:
    let moved = not reach[i.idx].contains(at)
    if moved: at = Raise(at.label - abs(d[i] - k))
    if moved == (d[i] >= k): group.push((grams: d[i], id: i.label))
    else: others.push((grams: d[i], id: i.label))
cook(group, k)
cook(others, k)
