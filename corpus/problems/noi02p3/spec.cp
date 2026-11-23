// noi02p3 Nine-Headed Dragon: the big head eats exactly k fruits including the root, every other
// head at least one; a branch costs when one head eats both ends. With three or more heads the
// small heads alternate freely, so only "big or small" matters per fruit: a tree knapsack over
// how many fruits the big head eats, merged child by child as a min-plus product. A `None` count
// is unreachable, and `min=` over lifted `+` skips it (R02.4.6).
let (n, heads, k) = read<(1..=300, 2..=300, 1..=300)>()
domain F = 1..=n
let g = Graph.undirected(read<[n - 1](u: F, v: F, cost: 0..=100_000)>())
enum Head = Small | Big
fn clash(w: Head, wc: Head) -> Bool = w == wc and (w == Head.Big or heads == 2)
// eat(v)[w][x]: the cheapest for v's subtree when head w eats v and the big head eats x of its
// fruits.
fn eat(v: F, from: ?F) -> [Head]List<?Int>:
    var best: [Head]List<?Int> = [Head] w => if w == Head.Big: [None, Some(0)] else: [Some(0), None]
    for a in g[v]:
        if Some(a.to) == from: continue
        let sub = eat(a.to, Some(v))
        let size = best[Head.Big].len + sub[Head.Big].len - 1
        var merged: [Head]List<?Int> = [Head] _ => List.filled(size, None)
        for w in Head:
            for wc in Head:
                for (x, above) in best[w].items:
                    for (y, below) in sub[wc].items:
                        merged[w][x + y] min= above + below + (if clash(w, wc): a.cost else: 0)
        best = merged
    best
print(if heads - 1 + k > n: -1 else: eat(F.first!, None)[Head.Big][k]!)
