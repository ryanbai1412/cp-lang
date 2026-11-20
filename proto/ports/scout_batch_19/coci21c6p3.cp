// COCI '21 Contest 6 #3, Naboj — ideal CP Lang sketch
// Category: Multi-answer Constructive Graph + Custom Checker (variable-length op sequence + -1 sentinel)
// Original C++: corpus/submissions/4530006.cpp (101 LOC raw, ~65 LOC stripped)
// Buildable today-form: proto/ports/scout_batch_19/today/coci21c6p3.cp (verified 80 random graphs vs orig C++)

let n: 1..=2e5 = read()
let m: 0..=2e5 = read()
domain V = 1..=n
let edges: [0..m](V, V) = read()

// Peel vertices whose remaining edges all point one way (in_deg == 0 or out_deg == 0);
// applying operations in reverse peeling order guarantees the final touch on every edge (u, v)
// either sets u -> 1 (outward) or v -> 0 (inward).
fn solve_naboj() -> ?List<(V, 0..=1)>:
    var out_adj: [V]List<V> = []
    var in_adj: [V]List<V> = []
    var in_deg: [V]Int = 0
    var out_deg: [V]Int = 0
    for (u, v) in edges:
        out_adj[u].push(v)
        in_adj[v].push(u)
        out_deg[u] += 1
        in_deg[v] += 1

    var vis: [V]Bool = false
    var q: List<V> = [u for u in V if in_deg[u] == 0 or out_deg[u] == 0]
    for u in q: vis[u] = true

    var ops: List<(V, 0..=1)> = []
    while let u = q.pop():
        let dir: 0..=1 = if in_deg[u] > 0: 0 elif out_deg[u] > 0: 1 else: 0
        ops.push((u, dir))
        for w in out_adj[u]:
            in_deg[w] -= 1
            if not vis[w] and in_deg[w] == 0:
                vis[w] = true
                q.push(w)
        for w in in_adj[u]:
            out_deg[w] -= 1
            if not vis[w] and out_deg[w] == 0:
                vis[w] = true
                q.push(w)

    if any(not vis[u] for u in V): return none
    ops.reversed()

match solve_naboj():
    none => print(-1)
    some(ops) =>
        print(ops.len)
        for (u, dir) in ops: print(u, dir)

// WISH B19-7: variable-length multi-answer constructive `checker:` simulating every op forward
// plus feasibility cross-check (impossible iff graph has a non-empty core with min in/out-deg >= 1).
checker:
    let k: -1..=200_000 = read_output()
    if k == -1:
        // Cross-check against independent feasibility predicate (peeling / brute-force spec)
        require not is_peelable(V, edges), "printed -1 on a solvable directed graph"
    else:
        let ops: [0..k](V, 0..=1) = read_output()
        // Track last operation timestamp and direction for each vertex
        var last_t: [V]Int = 0
        var last_d: [V]0..=1 = 0
        for (t, (u, d)) in ops.enumerate():
            last_t[u] = t + 1
            last_d[u] = d
        for (u, v) in edges:
            require max(last_t[u], last_t[v]) > 0, f"edge ({u}, {v}) never oriented"
            let ok = if last_t[u] > last_t[v]: last_d[u] == 1 else: last_d[v] == 0
            require ok, f"edge ({u}, {v}) oriented backwards"
