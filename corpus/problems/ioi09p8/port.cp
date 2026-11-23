// Source: IOI 2009 (Salesman), https://dmoj.ca/problem/ioi09p8
// Problem: n <= 5e5 fairs on a river, fair k on day T_k at location L_k (<= 500001, all distinct)
//   with profit M_k. Starting and ending at home S, moving upstream (towards smaller locations)
//   costs U per metre and downstream D (D <= U <= 10); fairs must be visited in day order, any
//   order within a day. Print the largest profit minus travel cost.
// Solution: best[x] = most money standing at x. Arriving at x from y above costs D (x - y) and
//   from below U (y - x), so two max SegTrees over locations hold best[y] + D y and
//   best[y] - U y, and reach(x) is two range folds. Within a day the route is monotone: sweep
//   the day's fairs once downstream and once upstream, then store the better of the two at each
//   fair. The answer is reach(home); O((n + L) log L).
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
    var from_above = SegTree<Loc, Int>(max, fill: never)  // best[y] + down * y
    var from_below = SegTree<Loc, Int>(max, fill: never)  // best[y] - up * y

    fn reach(x: Loc) -> Int:
        max(
            from_above.fold(Loc.first!..=x) - down * x.label,
            from_below.fold(x..=Loc.last!) + up * x.label,
        )

    fn stand(x: Loc, money: Int):
        from_above.set(x, max(from_above.fold(x..=x), money + down * x.label))
        from_below.set(x, max(from_below.fold(x..=x), money - up * x.label))

    stand(home, 0)
    let order = fairs.sorted(by: f => (f.day, f.at))
    var i = 0
    while i < n:
        var j = i
        while j < n and order[j].day == order[i].day: j += 1
        // Fairs i..j share a day, sorted by location.
        let g = j - i
        let arrive = [reach(order[i + t].at) for t in 0..g]
        var going_down = arrive.clone()
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
