// CCO '24 P4 — Infiltration / Tree Rendezvous (today form for proto/cpc)
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
            if sched.len > 0 and ty == 1 and sched[sched.len - 1] == 0:
                sched[sched.len - 1] = 1
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
