// IOI 2015 "Sorting": Ermek swaps two positions each round, then we swap two;
// sort the array within as few rounds as possible. After r rounds Ermek's
// swaps alone leave some arrangement; it needs (n - cycles) value swaps, and r
// rounds suffice iff that is at most r. Our swaps exchange values, so they are
// planned on the final arrangement and replayed at the values' current spots.
use std.seq.{perm_cycles}

@grader
fn find_swap_pairs(start: List<Int>, ermek: List<(Int, Int)>) -> List<(Int, Int)>:
    domain Pos = [start.len]
    let first: [Pos]Pos = [Pos] p => Pos(start[Int(p)])
    let moves = ermek.map(e => (Pos(e.0), Pos(e.1))).collect()

    fn plan(r: Int) -> List<(Pos, Pos)>:
        var p = first.clone()
        for (x, y) in moves.take(r):
            p.swap_at(x, y)
        [(c[0], v) for c in perm_cycles(p) for v in c.drop(1)]

    let rounds = (0..=moves.len).first_true(r => plan(r).len <= r)!
    let swaps = plan(rounds)
    var now = first.clone()
    var spot = now.inverse()
    var picks = List<(Int, Int)>()
    for (t, (x, y)) in moves.take(rounds).items:
        now.swap_at(x, y)
        (spot[now[x]], spot[now[y]]) = (x, y)
        let (u, v) = swaps.get(t) ?? (now[Pos.first!], now[Pos.first!])
        let (pu, pv) = (spot[u], spot[v])
        picks.push((Int(pu), Int(pv)))
        now.swap_at(pu, pv)
        (spot[u], spot[v]) = (pv, pu)
    picks
