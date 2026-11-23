// Source: CCO 2024 Day 2 P1 (Infiltration), https://dmoj.ca/problem/cco24p4
// Problem: output-only strategy on a tree of N = 100 rooms (N - 1 edges given). Two agents
//   start in different unknown rooms u, v; Ondrej may step to a neighbour on odd minutes,
//   Edward on even ones. Print T <= 1440, then each agent's room at times 1..T for every
//   start room, so they always meet; f(u, v) = first common time. Partial score: the
//   smaller max f(u, v) / dist(u, v), the better (full marks at <= 15).
// Solution: both agents only ever climb toward a centre root (depth <= 50), on one fixed
//   schedule of phases where one climbs while the other waits: Edward 3, Ondrej 11, Edward
//   22, Ondrej 10, Edward 25, Ondrej 25, Edward 30, Ondrej 30 steps (constants tuned in the
//   C++). Each has over 50 climbs, so they meet at the root at worst; the phases make the
//   lower one catch the other early. An idle minute is reused when the turn switches.
//   O(N^2) to find the root, O(N T) output.
// original: corpus/submissions/6435484.cpp
let n: 1..=100 = read()

if n == 100:
    domain V = 0..n
    var adj: [V]List<Int> = []
    for _ in 0..n - 1:
        let u: Int = read()
        let v: Int = read()
        adj[V(u)].push(v)
        adj[V(v)].push(u)

    var par: [V]Int = -1
    var dep: [V]Int = 0

    fn dfs_tree(u: Int, p: Int, d: Int):
        par[V(u)] = p
        dep[V(u)] = d
        for idx in 0..adj[V(u)].len:
            let v = adj[V(u)][idx]
            if v != p: dfs_tree(v, u, d + 1)

    var rt = -1
    for r in 0..n:
        dfs_tree(r, -1, 0)
        var mx = 0
        for i in 0..n:
            if dep[V(i)] > mx: mx = dep[V(i)]
        if mx <= 50: rt = r

    dfs_tree(rt, -1, 0)

    let gaps: List<Int> = [-3, 11, -22, 10, 25, -25, -30, 30]
    var strat_ty: List<Int> = []
    var strat_cnt: List<Int> = []
    for gi in 0..gaps.len:
        let nx = gaps[gi]
        if nx > 0:
            strat_ty.push(0)
            strat_cnt.push(nx)
        elif nx < 0:
            strat_ty.push(1)
            strat_cnt.push(-nx)

    var sched: List<Int> = []
    for si in 0..strat_ty.len:
        let ty = strat_ty[si]
        let cnt = strat_cnt[si]
        for _ in 0..cnt:
            if sched.len > 0 and ty == 1 and sched.last! == 0:
                sched.last! = 1
            else:
                sched.push(1 - ty)
                sched.push(ty)

    print(sched.len)
    for ty in 0..2:
        for start in 0..n:
            var u = start
            var row: List<Int> = []
            for j in 0..sched.len:
                if j % 2 == ty and sched[j] == 1 and u != rt:
                    u = par[V(u)]
                row.push(u)
            print(row)
