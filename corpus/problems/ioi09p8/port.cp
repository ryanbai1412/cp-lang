// IOI 2009 Salesman: fairs along a river, each at a location on a given day
// with a profit. The salesman starts and ends at home; moving upstream
// (towards smaller locations) costs `up` per metre and downstream `down`.
// Visit fairs in day order for the largest total profit.
// best[x] = most money when standing at x. Arriving at x from above costs
// down * (x - y) and from below up * (y - x), so two max-trees hold
// best[y] + down * y and best[y] - up * y. Fairs on one day are swept once
// downstream and once upstream: within a day the route is monotone.
// original: corpus/submissions/4811954.cpp
let n: 1..=500000 = read()
let up: 1..=10 = read()
let down: 1..=10 = read()
domain Loc = 1..=500001
let home: Loc = read()

struct Fair:
    day: 1..=500000
    at: Loc
    profit: 1..=4000
let fairs: [n]Fair = read()

fn most() -> Int:
    let never = -1e18
    var from_above = SegTree<Loc, Int>(never, (a, b) => max(a, b))  // best[y] + down * y
    var from_below = SegTree<Loc, Int>(never, (a, b) => max(a, b))  // best[y] - up * y

    fn reach(x: Loc) -> Int:
        max(
            from_above.query(Loc.first, x) - down * x.label,
            from_below.query(x, Loc.last) + up * x.label,
        )

    fn stand(x: Loc, money: Int):
        from_above.set(x, max(from_above.query(x, x), money + down * x.label))
        from_below.set(x, max(from_below.query(x, x), money - up * x.label))

    stand(home, 0)
    let order = fairs.sorted(by: f => (f.day, f.at))
    var i = 0
    while i < n:
        var j = i
        while j < n and order[j].day == order[i].day: j += 1
        // Fairs i..j share a day, sorted by location.
        let g = j - i
        let arrive = [reach(order[i + t].at) for t in 0..g]
        var going_down = arrive
        var going_up = arrive
        for t in 0..g:
            if t > 0:
                let gap = order[i + t].at.label - order[i + t - 1].at.label
                going_down[t] max= going_down[t - 1] - down * gap
            going_down[t] += order[i + t].profit
        for t in (0..g).rev:
            if t + 1 < g:
                let gap = order[i + t + 1].at.label - order[i + t].at.label
                going_up[t] max= going_up[t + 1] - up * gap
            going_up[t] += order[i + t].profit
        for t in 0..g: stand(order[i + t].at, max(going_down[t], going_up[t]))
        i = j
    reach(home)

print(most())
