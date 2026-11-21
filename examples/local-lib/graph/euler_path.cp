//! Eulerian trails: a walk that uses every edge exactly once (Hierholzer's algorithm).
use std.seq.{reverse}

/// The edges of a walk that uses each of `edges` exactly once, as edge indices in walking
/// order, or `none` when no such walk exists. `directed` says whether `(u, v)` may only be
/// walked from `u` to `v`. The walk is closed whenever every vertex is balanced. O(V + E).
pub fn euler_path<V>(edges: List<(V, V)>, directed: Bool) -> ?List<Int>:
    if edges.is_empty: return []
    var adj: [V]List<(Int, V)> = [V] _ => []
    var surplus: [V]Int = 0
    for i in 0..edges.len:
        let (u, v) = edges[i]
        adj[u].push((i, v))
        if directed:
            surplus[u] += 1
            surplus[v] -= 1
        else:
            adj[v].push((i, u))
    var start = edges[0].0
    var odd = 0
    for v in V:
        let extra = if directed: surplus[v] else: adj[v].len % 2
        if directed and extra < -1 or extra > 1: return none
        if extra == 1:
            odd += 1
            start = v
        elif directed and extra == -1: odd += 1
    if odd > 2: return none
    var next: [V]Int = 0
    var used = List.filled(edges.len, false)
    var stack: List<(V, ?Int)> = [(start, none)]
    var walk: List<Int> = []
    while let (v, came) = stack.last:
        var moved = false
        while next[v] < adj[v].len and not moved:
            let (i, w) = adj[v][next[v]]
            next[v] += 1
            if not used[i]:
                used[i] = true
                stack.push((w, i))
                moved = true
        if not moved:
            stack.pop()
            if let i = came: walk.push(i)
    if walk.len != edges.len: return none
    reverse(mut walk)
    walk

@test
fn walks_are_valid_and_exist_exactly_when_brute_force_finds_one():
    domain V = 0..4
    for _ in 0..200:
        let directed = rand(0, 1) == 1
        let edges = [(V.nth(rand(0, 3)), V.nth(rand(0, 3))) for _ in 0..rand(0, 6)]
        let got = euler_path(edges, directed)
        assert (got != none) == has_walk(edges, directed)
        if let walk = got:
            assert walk.sorted() == List(0..edges.len)
            assert any(follows(edges, walk, directed, s) for s in V)

/// Does `walk` trace connected edges, starting at `start`?
fn follows<V>(edges: List<(V, V)>, walk: List<Int>, directed: Bool, start: V) -> Bool:
    var at = start
    for i in walk:
        let (u, v) = edges[i]
        if u == at: at = v
        elif not directed and v == at: at = u
        else: return false
    true

fn has_walk<V>(edges: List<(V, V)>, directed: Bool) -> Bool:
    var used = List.filled(edges.len, false)
    edges.is_empty or any(extend(edges, directed, mut used, v, 0) for v in V)

fn extend<V>(edges: List<(V, V)>, directed: Bool, mut used: List<Bool>, at: V, done: Int) -> Bool:
    if done == edges.len: return true
    for i in 0..edges.len:
        if used[i]: continue
        let (u, v) = edges[i]
        var to: ?V = none
        if u == at: to = v
        elif not directed and v == at: to = u
        if let w = to:
            used[i] = true
            if extend(edges, directed, mut used, w, done + 1): return true
            used[i] = false
    false
