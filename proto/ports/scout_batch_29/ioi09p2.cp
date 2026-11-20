// IOI 2009 Day 1 P2, Hiring: n candidates, each with a minimum salary s
// and a qualification q, and a budget w. Everyone hired is paid in
// proportion to q and at least their own s. Hire as many as possible,
// and among those as cheaply as possible; print the count and the hired
// candidates, one per line (any best set is accepted).
//
// With pay rate s/q per qualification point set by the most demanding
// hire, the cheapest k hires are the k smallest q among the candidates
// that accept the rate. Sweep the rate upwards and keep those in a
// max-heap, dropping the largest q while the bill is over budget.
// original: corpus/submissions/5102705.cpp
use std.heap.{Heap}
use std.algebra.{Order}

let (n, w): (Int, Int) = read()
domain Worker = 1..=n
let ask: [Worker](s: Int, q: Int) = read()

// Rates compare exactly by cross-multiplying: s and q are at most 2e4.
var by_rate = [i for i in Worker]
sort(mut by_rate, by: Order<Worker>((a, b) => ask[a].s * ask[b].q < ask[b].s * ask[a].q))

type Plan = (setter: Worker, size: Int, points: Int)

/// The bill of `a` is below the bill of `b`: points * s / q, cross-multiplied.
fn cheaper(a: Plan, b: Plan) -> Bool:
    a.points * ask[a.setter].s * ask[b.setter].q < b.points * ask[b.setter].s * ask[a.setter].q

/// Sweeps the rates. Without `stop`, returns the best plan: most hires,
/// then the smallest bill. With `stop`, prints the hires once that
/// worker has set the rate.
fn sweep(stop: ?Worker) -> ?Plan:
    var hired = Heap<(Int, Worker)>(Order<(Int, Worker)>((a, b) => a.0 > b.0))
    var points = 0
    var best: ?Plan = none
    for i in by_rate:
        hired.push((ask[i].q, i))
        points += ask[i].q
        while points * ask[i].s > w * ask[i].q:
            points -= hired.pop()!.0
        let plan: Plan = (setter: i, size: hired.len, points: points)
        if best == none or plan.size > best!.size or (plan.size == best!.size and cheaper(plan, best!)):
            best = plan
        if stop == i:
            print(hired.len)
            while not hired.is_empty: print(hired.pop()!.1)
            return best
    best

let best = sweep(none)!
sweep(best.setter)
