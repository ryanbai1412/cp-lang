// Source: IOI 2009 Day 1 P2 (Hiring), https://dmoj.ca/problem/ioi09p2
// Problem: n <= 5e5 candidates, candidate k wants at least S_k and has qualification Q_k
//   (both <= 2e4); budget W <= 1e10. Hired workers are paid in proportion to Q, each at
//   least their S, total at most W. Hire as many as possible, then as cheaply as possible;
//   print the count and the hired candidates (any best set).
// Solution: the rate per point of Q is set by the hire with the largest S/Q, and the
//   cheapest hires at that rate are those with the smallest Q. Sweep candidates by the exact
//   `Ratio` S/Q, keep a max-heap of Q, and pop the largest Q while S * points > W * Q; keep
//   the best plan (most hires, then the smallest bill, compared by cross-multiplying). Its hires are the
//   plan's size smallest Q among the candidates up to its rate setter. O(n log n).
// limits: 1500 ms, 64 MB
// original: corpus/submissions/5102705.cpp
// Bounds: n: 1..=5e5; w: 1..=1e10; ask.s, ask.q: 1..=2e4
let (n, w): (1..=5e5, 1..=1e10) = read()
domain Worker = 1..=n
let ask: [Worker](s: 1..=2e4, q: 1..=2e4) = read()

let rate: [Worker]Ratio = [Worker] i => Ratio.new(ask[i].s, ask[i].q)
let by_rate = Worker.sort(by: i => rate[i])

// A plan hires `size` candidates, the rate setter `by` among them, with `points` of Q in all;
// it bills by.s * points / by.q.
type Plan = (size: Int, by: Worker, points: Int, upto: Int)
fn bill_times(a: Plan, q: Int) -> Int = ask[a.by].s * a.points * q
fn better(a: Plan, b: Plan) -> Bool =
    a.size > b.size or
        (a.size == b.size and bill_times(a, ask[b.by].q) < bill_times(b, ask[a.by].q))

var hired = MaxHeap<Int>()
var points = 0
var best: ?Plan = None
for (upto, i) in by_rate.items:
    hired.push(ask[i].q)
    points += ask[i].q
    while ask[i].s * points > w * ask[i].q: points -= hired.pop()!
    let plan: Plan = (size: hired.len, by: i, points:, upto:)
    if best == None or better(plan, best!): best = plan

let plan = best!
let hires = by_rate[..=plan.upto].sorted(by: i => ask[i].q).take(plan.size).collect()
print(hires.len)
for i in hires: print(i)
