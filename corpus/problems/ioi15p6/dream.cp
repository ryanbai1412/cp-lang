// IOI 2015 "Towns": a tree's leaves are towns; query leaf distances (cached)
// to find the hub radius R, and its sign says whether some hub is balanced.
//
// stdin as the sample grader: the subtask and T, then per case N and the N x N
// distances; getDistance answers from them and traps past the subtask's limit.
let (sub, cases) = read<(1..=6, 1..=40)>()
for _ in 0..cases:
    let n = read<6..=110>()
    domain T = [n]
    let dist = read<[T, T]0..=1e6>()
    let limit = match sub:
        1 | 3 => n * (n - 1) / 2
        5 => 5 * n
        _ => (7 * n + 1) / 2
    var asked = 0
    fn getDistance(i: T, j: T) -> Int:
        asked += 1
        assert asked <= limit, "query {asked} is past the limit {limit}"
        dist[i, j]

    @memo(map)
    fn known(i: T, j: T) -> Int = getDistance(i, j)
    fn d(u: T, v: T) -> Int = if u == v: 0 else: known(min(u, v), max(u, v))

    fn hub_distance() -> Int:
        let o = T.first!
        let a = T.max(by: v => d(o, v))!
        let b = T.max(by: v => d(a, v))!
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
        let r = T.map(i => max(at[i].along, diam - at[i].along)).min()!
        var before = 0
        var hubs = List<Int>()
        for (h, c) in List(T.tally(by: i => at[i].along).items).sorted():
            if max(h, diam - h) == r:
                let after = n - before - c
                if max(before, c, after) <= n / 2:
                    return r
                if max(before, after) <= n / 2:
                    hubs.push(h)
            before += c
        let h = hubs.first ?? return -r
        let same = (u: T, v: T) => d(u, v) != at[u].down + at[v].down
        let branch = T.filter(i => at[i].along == h)
        if branch.majority(eq: same).is_some_and(m => m.count > n / 2): -r else: r

    print(hub_distance())
