// IOI 2015 "Towns": a tree's leaves are towns; query leaf distances (cached)
// to find the hub radius R, and its sign says whether some hub is balanced.
// Every town hangs off the diameter a..b at `along`; a hub is a point of the
// diameter minimizing the farther end. A hub is balanced when no side holds
// more than n / 2 towns, and the towns at the hub split into branches told
// apart only by distance queries, so a majority vote decides it.
//
// The task is a grader function; this reads the sample grader's input (the
// subtask and T, then per case N and the N x N distances), answers each
// getDistance from it, traps past the subtask's query limit, and prints R or -R.
// copies: 1
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
    fn d(u: T, v: T) -> Int = if u == v: 0 else: getDistance(min(u, v), max(u, v))

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
        // Fischer-Salzberg majority over the hub's towns: at most 3k / 2 queries,
        // where Boyer-Moore's two passes take 2k and break the 7n / 2 limit.
        // `closed` runs hold a lead's run (as long as `others`) and towns unlike it.
        let same = (u: T, v: T) => d(u, v) != at[u].down + at[v].down
        var closed = List<(lead: T, others: List<T>)>()
        var lead: ?T = None
        var run = 0
        var others = List<T>()
        for i in T.filter(i => at[i].along == h):
            if let Some(l) = lead:
                if same(l, i): run += 1 else: others.push(i)
            else:
                lead = Some(i)
                run = 1
            if run == others.len:
                closed.push((lead: lead!, others:))
                lead = None
                run = 0
                others = List()
        let c = lead ?? return r
        for s in closed:
            run += if same(s.lead, c): s.others.len else: s.others.count(x => same(x, c))
        if run > n / 2: -r else: r

    print(hub_distance())
