// IOI 2015 "Towns": a tree's leaves are towns; query leaf distances (cached)
// to find the hub radius R, and its sign says whether some hub is balanced.
@grader fn getDistance(i: Int, j: Int) -> Int

pub fn hub_distance(n: Int, sub: Int) -> Int:
    domain T = [n]
    @memo
    fn d(u: T, v: T) -> Int = if u == v: 0 else: getDistance(Int(min(u, v)), Int(max(u, v)))

    let o = T.first!
    let a = T.max_by(v => d(o, v))!
    let b = T.max_by(v => d(a, v))!
    let diam = d(a, b)
    let o_down = (d(a, o) + d(b, o) - diam) / 2
    let o_along = d(a, o) - o_down

    fn place(i: T) -> (along: Int, down: Int):
        let shared = (d(a, i) + d(o, i) - d(o, a)) / 2
        let (from_o, from_a) = (d(o, i) - shared, d(a, i) - shared)
        if from_o <= o_down: (along: o_along, down: shared + from_a - o_along)
        elif from_a < o_along: (along: from_a, down: shared)
        else: (along: d(a, i), down: 0)

    let at = [T] i => place(i)
    let r = T.min(of: i => max(at[i].along, diam - at[i].along))!
    var before = 0
    var hubs = List<Int>()
    for (h, c) in T.counts(by: i => at[i].along).items:
        if max(h, diam - h) == r:
            let after = n - before - c
            if max(before, c, after) <= n / 2:
                return r
            if max(before, after) <= n / 2:
                hubs.push(h)
        before += c
    let h = hubs.first ?? return -r
    let branch = T.filter(i => at[i].along == h).collect()
    let same = (u: T, v: T) => d(u, v) != at[u].down + at[v].down
    if branch.majority(eq: same).is_some_and(m => m.count > n / 2): -r else: r
