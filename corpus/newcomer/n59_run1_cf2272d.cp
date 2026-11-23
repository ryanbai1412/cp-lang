// First draft from acceptance run 1 (cold agent, docs only), Codeforces 2272D, timed round.
@!multitest
let k: 2..=2e5 = read()
var hub_tree = 1
var hub = 1
var found = false  // expect: missing flag-for-first-match
for t in 1..=k:
    let n: 1..=2e5 = read()
    var adj: [1..=n]List<Int> = []  // expect: manual_adjacency_list
    for _ in 0..n - 1:
        let (u, v): (1..=n, 1..=n) = read()
        adj[u].push(v)
        adj[v].push(u)  // expect: symmetric_write
    if n >= 2 and not found:
        var parent: [1..=n]Int = 0
        var order: List<Int> = [1]
        var j = 0  // expect: missing manual-queue-head-index
        while j < order.len:
            let u = order[j]
            j += 1
            for v in adj[u] if v != parent[u]:
                parent[v] = u
                order.push(v)
        var blocked: [1..=n]Bool = false
        for u in order.rev:
            if blocked[u]:
                hub = u
            elif parent[u] != 0:
                blocked[parent[u]] = true
        hub_tree = t
        found = true
for t in 1..=k if t != hub_tree:
    print(t, 1, hub_tree, hub)
