// NOI 2020 Day 2 P1, Dishes: n ingredients with d[i] grams each, to be
// used up by m dishes of exactly k grams, each dish made of one or two
// ingredients (sum of d = m k, and m >= n - 2). Print for each dish
// `i x` or `i x j y` (ingredient and grams), or -1 if it cannot be done.
// Any valid plan is accepted.
//
// With m >= n - 1 a greedy always works: pair the scarcest ingredient
// with the most plentiful one. With m = n - 2 the ingredients must split
// into two groups that are each one dish short of their size, that is,
// a subset whose d[i] - k add up to -k: a subset sum over a bitset.
// original: corpus/submissions/3890337.cpp
use std.bitset.{Bitset}

type Stock = (grams: Int, id: Int)

/// Cooks dishes of `k` grams from `stock` until it is used up. Needs
/// at least (ingredients - 1) dishes' worth, which the greedy keeps true.
fn cook(mut stock: List<Stock>, k: Int):
    while stock.len > 0:
        var (low, high) = (0, 0)
        for i in 0..stock.len:
            if stock[i] < stock[low]: low = i
            if stock[i] > stock[high]: high = i
        let (a, b) = (stock[low], stock[high])
        if a.grams >= k:
            print(a.id, k)
            stock[low] = (grams: a.grams - k, id: a.id)
        else:
            assert a.grams + b.grams >= k
            print(a.id, a.grams, b.id, k - a.grams)
            stock[low] = (grams: 0, id: a.id)
            stock[high] = (grams: b.grams - (k - a.grams), id: b.id)
        stock = [s for s in stock if s.grams > 0]

fn solve():
    let (n, m, k): (Int, Int, Int) = read()
    domain Item = 1..=n
    let d: [Item]Int = read()
    if m >= n - 1:
        var all: List<Stock> = []
        for i in Item: all.push((grams: d[i], id: i.label))
        cook(mut all, k)
        return

    // Start from "every ingredient below k is in the group": the group
    // sum of d[i] - k is then `lowest`, and moving ingredient i in or out
    // of the group raises it by |d[i] - k|. Reach -k.
    let lowest = sum(min(d[i] - k, 0) for i in Item)
    let highest = sum(max(d[i] - k, 0) for i in Item)
    domain Raise = 0..=(highest - lowest)
    let goal = Raise.try(-k - lowest)
    var reach: List<Bitset<Raise>> = [Bitset<Raise>.new()]     // reach[j]: after the first j items
    reach[0].add(Raise(0))
    for i in Item:
        var next = reach[i.ord].shifted(abs(d[i] - k))
        next.union(reach[i.ord])
        reach.push(next)
    if goal == none or not reach[n].contains(goal!):
        print(-1)
        return

    var at = goal!
    var (group, others): (List<Stock>, List<Stock>) = ([], [])
    for i in Item.rev:
        let moved = not reach[i.ord].contains(at)       // item i was moved in or out
        if moved: at = at.offset(-abs(d[i] - k))!
        if moved == (d[i] >= k): group.push((grams: d[i], id: i.label)) else: others.push((grams: d[i], id: i.label))
    cook(mut group, k)
    cook(mut others, k)

let t: Int = read()
for _ in 0..t: solve()
