// Source: CCO 2022 P2 (Rainy Markets), https://dmoj.ca/problem/cco22p2
// Problem: n <= 1e6 bus stops hold B_i <= 2e9 people; market i, between stops i and i + 1, has
//   P_i <= 1e9 people and U_i <= 1e9 umbrellas at cost 1. Each person goes to stop i, to stop
//   i + 1 or buys an umbrella. Print NO if someone gets wet, else YES, the least money spent and
//   per market how many go left, buy, go right.
// Solution: greedy left to right: market i fills what is left of stop i, then stop i + 1, and the
//   rest buy umbrellas. If they run out, earlier markets with spare umbrellas buy instead and
//   each market between shifts that many people from its right stop to its left, freeing room
//   in stop i. A stack keeps the spare umbrellas, nearest on top, truncated to the right-goers z
//   since every shifted unit passes through them; shifts go into a difference array over the
//   markets between, applied at the end. O(n) amortized.
// original: corpus/submissions/4801882.cpp
let n: 2..=1e6 = read()
domain Stop = 0..n
domain Market = 0..n - 1
let room: [Stop]0..=2e9 = read()
let people: [Market]0..=1e9 = read()
let umbrellas: [Market]0..=1e9 = read()

struct Plan:
    var left: Int
    var buy: Int
    var right: Int

fn stay_dry() -> ?[Market]Plan:
    var plan: [Market]Plan = Plan(0, 0, 0)
    var spare: List<(at: Market, count: Int, below: Int)> = []
    var total = 0
    var shifted = Diff<Market, Int>()
    var came = 0
    for i in Market:
        var left = min(people[i], room[Stop(i.ord)] - came)
        let right = min(room[Stop(i.ord + 1)], people[i] - left)
        var buy = people[i] - left - right
        if buy <= umbrellas[i]:
            spare.push((at: i, count: umbrellas[i] - buy, below: total))
            total += umbrellas[i] - buy
        else:
            let need = buy - umbrellas[i]
            if need > total: return None
            total -= need
            var owed = need
            while owed > 0:
                let (j, count, _) = spare.last!
                let take = min(owed, count)
                owed -= take
                plan[j].buy += take
                plan[j].right -= take
                shifted.add(j.r..i.l, take)
                if take == count: spare.pop()
                else: spare.last!.count -= take
            left += need
            buy -= need
        plan[i] = Plan(left, buy, right)
        came = right
        if total > right:
            while let Some(top) = spare.last and top.below >= right: spare.pop()
            if let Some(top) = spare.last: spare.last!.count = right - top.below
            total = right
    let moved = shifted.finish()
    for i in Market:
        plan[i].left += moved[i]
        plan[i].right -= moved[i]
    plan

if let plan = stay_dry():
    print("YES")
    print(Market.map(i => plan[i].buy).sum())
    for p in plan: print(p.left, p.buy, p.right)
else:
    print("NO")
