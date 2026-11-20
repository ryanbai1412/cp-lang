// IOI 2014 P3, Game — ideal CP Lang sketch
// Category: Reversed-control Adaptive IOI Interactive (judge calls solution's `hasEdge` n*(n-1)/2 times!)
// Original C++: corpus/submissions/4683972.cpp (68 LOC raw, ~42 LOC stripped)
// Buildable today-form: proto/ports/scout_batch_19/today/ioi14p3.cp (stdio-adapted, verified)

// WISH B19-5: Stateful exported module / class for reversed-control callback graders.
// Unlike standard interactive problems where the solution drives `ask(...)`, here the JUDGE
// drives the loop and calls `initialize(n)` once followed by `has_edge(u, v)` n*(n-1)/2 times.
export grader Game:
    domain V = 0..n
    var dsu = Dsu(V)
    var asked: [V, V]Int = 0

    fn initialize(n: 2..=1500):
        // Domain V, dsu, and asked initialize from `n`
        pass

    fn has_edge(u0: V, v0: V) -> 0..=1:
        let u = dsu.find(u0)
        let v = dsu.find(v0)
        asked[u, v] += 1
        asked[v, u] += 1
        if asked[u, v] == dsu.size(u) * dsu.size(v):
            let root = dsu.merge(u, v)
            let other = if root == u: v else: u
            for i in V:
                asked[root, i] += asked[other, i]
                if i != root:
                    asked[i, root] += asked[i, other]
            return 1
        0

// WISH B19-5: Driver-mode `interactor:` — drives the exported callbacks in an arbitrary (or adaptive!)
// order and checks the game's invariants after EVERY step, not only at the end!
interactor:
    gen (n, order):
        let n = rng.int(2..=min(size, 1500))
        let pairs = [(u, v) for u in 0..n for v in (u + 1)..n]
        yield (n, rng.shuffled(pairs))

    run(sol: Game):
        sol.initialize(n)
        var yes = Dsu(0..n)
        var edges_yes = 0
        for (idx, (u, v)) in order.enumerate():
            let ans = sol.has_edge(u, v)
            if ans == 1:
                require yes.find(u) != yes.find(v), "redundant cycle edge revealed"
                yes.merge(u, v)
                edges_yes += 1
            // Must stay strictly ambiguous (disconnected) before the very last query!
            if idx + 1 < order.len:
                require yes.components > 1, "graph became connected before last query"
        require yes.components == 1 and edges_yes == n - 1
