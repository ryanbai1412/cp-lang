// Source: IOI 2009 Day 1 P2 (Hiring), https://dmoj.ca/problem/ioi09p2
// Problem: n <= 5e5 candidates, candidate k wants at least S_k and has qualification Q_k
//   (both <= 2e4); budget W <= 1e10. Hired workers are paid in proportion to Q, each at
//   least their S, total at most W. Hire as many as possible, then as cheaply as possible;
//   print the count and the hired candidates (any best set).
// Solution: the rate per point of Q is set by the hire with the largest S/Q, and the
//   cheapest hires at that rate are those with the smallest Q. Sweep candidates by S/Q
//   (exact cross-multiplied order), keep a max-heap of Q, and pop the largest Q while
//   points * rate is over budget; keep the best (count, then bill) plan. A second sweep
//   stops at the best plan's rate setter and prints its heap. O(n log n).
// limits: 1500 ms, 64 MB
// original: corpus/submissions/5102705.cpp

let (n, w): (Int, Int) = read()
domain Worker = 1..=n
let ask: [Worker](s: Int, q: Int) = read()

struct Rate(w: Worker, s: Int, q: Int)
impl Ord for Rate:
    fn lt(a, b) = a.s * b.q < b.s * a.q

// Rates compare exactly by cross-multiplying: s and q are at most 2e4.
var by_rate = [Rate(i, ask[i].s, ask[i].q) for i in Worker]
by_rate.sort()

type Plan = (setter: Worker, size: Int, points: Int)

/// The bill of `a` is below the bill of `b`: points * s / q, cross-multiplied.
fn cheaper(a: Plan, b: Plan) -> Bool:
    a.points * ask[a.setter].s * ask[b.setter].q < b.points * ask[b.setter].s * ask[a.setter].q

/// Sweeps the rates. Without `stop`, returns the best plan: most hires,
/// then the smallest bill. With `stop`, prints the hires once that
/// worker has set the rate.
fn sweep(stop: ?Worker) -> ?Plan:
    var hired = MaxHeap<(Int, Worker)>()
    var points = 0
    var best: ?Plan = None
    for r in by_rate:
        let i = r.w
        hired.push((ask[i].q, i))
        points += ask[i].q
        while points * ask[i].s > w * ask[i].q:
            points -= hired.pop()!.0
        let plan: Plan = (setter: i, size: hired.len, points:)
        if best == None or plan.size > best!.size or (
            plan.size == best!.size and cheaper(plan, best!)
        ):
            best = plan
        if stop == i:
            print(hired.len)
            while not hired.is_empty: print(hired.pop()!.1)
            return best
    best

let best = sweep(None)!
let _ = sweep(best.setter)
