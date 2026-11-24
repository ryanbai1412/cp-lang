// Source: NOI 2009 (Transformed Sequence), https://dmoj.ca/problem/noi09p1
// Problem: given D[i] for i in 0..n (n <= 1e4), find a permutation T of 0..n with circular
//   distance min(|i - T[i]|, n - |i - T[i]|) = D[i] for every i; print the lexicographically
//   smallest one, or "No Answer".
// Solution: T[i] is (i - D[i]) mod n or (i + D[i]) mod n, so each position is an edge between
//   its two candidate values and T picks one endpoint per edge, each value once. A value of
//   degree 1 is forced: peel those. What is left must be disjoint cycles; going through
//   positions in order, the first unassigned edge of a cycle takes its smaller value, which
//   orients the whole cycle. O(n).
// original: corpus/submissions/5102817.cpp

let n: 1..=1e4 = read()
domain Pos = 0..n
domain Val = 0..n
let dist: [Pos]0..n = read()
let ends = [Pos] i => (u: Val((i.ord - dist[i]) % n), v: Val((i.ord + dist[i]) % n))
let g = Graph.undirected(ends)

fn solve() -> ?[Pos]Val:
    var at: [Pos]?Val = None
    var taken: [Val]Bool = false
    var deg = [Val] u => g[u].len
    fn take(i: Pos, u: Val) -> Bool:
        if at[i] != None or taken[u]: return false
        at[i] = u
        taken[u] = true
        true

    var forced = [u for u in Val if deg[u] == 1]
    while let u = forced.pop():
        for a in g[u]:
            if at[a.e] == None:
                if not take(a.e, u): return None
                deg[u] -= 1
                deg[a.to] -= 1
                if deg[a.to] == 1: forced.push(a.to)

    for i in Pos:
        if at[i] != None: continue
        let first = min(ends[i].u, ends[i].v)
        if not take(i, first): return None
        var cur = if first == ends[i].u: ends[i].v else: ends[i].u
        while cur != first:
            if let a = g[cur].find(a => at[a.e] == None):
                if not take(a.e, cur): return None
                cur = a.to
            else: return None
    if Val.any(u => not taken[u]): return None
    Some([Pos] i => at[i]!)

if let t = solve(): print(t) else: print("No Answer")
