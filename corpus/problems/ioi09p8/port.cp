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
// Bounds: n: 1..=500000; up, down: 1..=10; home: 1..=500001
let n: 1..=500000 = read()
let up: 1..=10 = read()
let down: 1..=10 = read()
domain Loc = 1..=500001
let home: Loc = read()

struct Fair:
    day: 1..=500000
    at: Loc
    profit: 1..=4000
var fairs: [n]Fair = read()

// best[y] is above -U * 5e5 for every y stood on and at most 4000 n; the empty fold is -1e10.
type Money = -1e10..=1e10
var from_above = SegTree<Loc, Money>(max)  // best[y] + down * y where y is stood on
var from_below = SegTree<Loc, Money>(max)  // best[y] - up * y

// Home is stood on from the start, so one side always reaches x with a real value.
fn reach(x: Loc) -> Int = max(from_above.fold(..=x) - down * x.label, from_below.fold(x..) + up * x.label)

fn stand(x: Loc, money: Int):
    from_above.at(x) max= money + down * x.label
    from_below.at(x) max= money - up * x.label

stand(home, 0)
fairs.sort(by: f => (f.day, f.at))
for day in fairs.runs(by: f => f.day):
    let stops = [(at: f.at, gain: reach(f.at) + f.profit, profit: f.profit) for f in day]
    let k = stops.len
    var going_down = [s.gain for s in stops]
    for t in 1..k:
        let gap = stops[t].at.label - stops[t - 1].at.label
        going_down[t] max= going_down[t - 1] - down * gap + stops[t].profit
    var going_up = [s.gain for s in stops]
    for t in (0..k - 1).rev:
        let gap = stops[t + 1].at.label - stops[t].at.label
        going_up[t] max= going_up[t + 1] - up * gap + stops[t].profit
    for t in 0..k: stand(stops[t].at, max(going_down[t], going_up[t]))
print(reach(home))
