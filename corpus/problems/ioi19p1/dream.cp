// IOI 2019 Shoes: adjacent swaps to arrange shoes as (left, right) pairs of equal size.
// Greedy from the left: the first unmatched shoe pairs with the nearest unmatched shoe of the
// opposite foot; moving it next to it costs the unmatched shoes in between, plus one if the
// first shoe is a right shoe.
fn count_swaps(s: List<Int>) -> Int:
    var queue: HashMap<Int, Deque<s.Key>> = {}
    for (i, x) in s.items: queue[x].push_back(i)
    var matched = Fenwick(+, [s.keys]0)
    var swaps = 0
    for (i, x) in s.items:
        if matched.at(i) == 1: continue
        queue[x].pop_front()
        let j = queue[-x].pop_front()!
        swaps += (i.r..j.l).len - matched.fold(i.r..j.l) + (if x > 0: 1 else: 0)
        matched.add(j, 1)
    swaps
