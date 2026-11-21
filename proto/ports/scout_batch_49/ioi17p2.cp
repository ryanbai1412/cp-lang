// IOI 2017 Day 1 P2, Wiring
// original: corpus/submissions/5724153.cpp
@grader
fn min_total_length(red: List<Int>, blue: List<Int>) -> Int:
    // every point as (position, +1 red / -1 blue), left to right
    let (P, pts) = ([(x, 1) for x in red] + [(x, -1) for x in blue]).sorted().freeze()

    // near[p]: distance to the closest point of the other colour
    // FRICTION: "nearest earlier element satisfying a predicate, from both sides" has no helper
    var near: [P]?Int = none
    fn sweep(order: Range<P>):
        var seen: [-1..=1]?Int = none            // last position of each colour
        for p in order:
            let (x, colour) = pts[p]
            if let y = seen[-colour]: near[p] min= abs(x - y)
            seen[colour] = x
    sweep(P.first..=P.last)
    sweep(P.rev)

    // cost[b]: cheapest wiring of the points left of boundary b. Either the
    // last point takes its own wire to the nearest other colour, or the block
    // since the last boundary with the same red-minus-blue balance is matched
    // like brackets, which costs exactly |sum of signed positions| over it.
    var cost: [P.bounds]Int = 0
    var last = HashMap<Int, (at: P.bounds, signed: Int)>()
    var (balance, signed) = (0, 0)
    last[0] = (P.first.l, 0)
    for p in P:
        let (x, colour) = pts[p]
        balance += colour
        signed += colour * x
        cost[p.r] = cost[p.l] + near[p]!
        if let seen = last.get(balance): cost[p.r] min= cost[seen.at] + abs(signed - seen.signed)
        last[balance] = (p.r, signed)
    cost[P.last.r]
