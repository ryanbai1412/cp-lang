// Source: Codeforces 2062E1 (The Game, Easy Version),
//   https://codeforces.com/contest/2062/problem/E1
// Problem: a tree of n <= 4e5 nodes rooted at 1, values w_i <= n. Players alternately delete the
//   subtree of a remaining node heavier than the last one chosen (first move: any node); the
//   first who cannot move wins. Print a node Cirno can pick first to win, or 0. Multitest.
// Solution: pick the heaviest v with a heavier node outside its subtree: any reply u has all
//   nodes heavier than u in its subtree, so after it nothing heavier is left. Scan weights
//   downward keeping the LCA of all strictly heavier nodes; the first v that is not an
//   ancestor of it wins. O(n log n).

@!multitest
let n: 1..=400_000 = read()
domain V = 1..=n
domain W = 1..=n
let w: [V]W = read()
let t = Rooted(Graph.undirected(read<[0..n - 1](V, V)>()), V.first!)
let by_w = buckets(w)

fn win() -> ?V:
    var heavier_lca: ?V = None
    for weight in W.rev:
        let b = by_w[weight]
        if let l = heavier_lca:
            if let v = b.find(v => not t.is_anc(v, l)): return v
        for v in b:
            heavier_lca = if let l = heavier_lca: t.lca(l, v) else: v
    None

if let v = win(): print(v) else: print(0)
