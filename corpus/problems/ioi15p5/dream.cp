// IOI 2015 "Sorting": Ermek swaps two positions each round, then we swap two;
// sort the array within Ermek's rounds.
pub fn find_swap_pairs(start: List<Int>, ermek: List<(Int, Int)>) -> List<(Int, Int)>:
    domain Pos = [start.len]
    let start = Perm<Pos>(start)
    let land = ermek.fold(start.clone(), (mut p, (x, y)) => p.swap(Pos(x), Pos(y))).inverse()
    let plan = land.sorting_swaps()
    var now = start
    var picks = List<(Int, Int)>()
    for (t, (x, y)) in ermek.items:
        now.swap(Pos(x), Pos(y))
        let (u, v) = plan.get(t) ?? (now[Pos.first!], now[Pos.first!])
        let (pu, pv) = (now.where(u), now.where(v))
        picks.push((Int(pu), Int(pv)))
        now.swap(pu, pv)
    picks
