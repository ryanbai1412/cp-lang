// Source: Codeforces 2137G (Cry Me a River), https://codeforces.com/contest/2137/problem/G
// limits: 2000 ms, 256 MB
// Problem: a DAG with n, m <= 2e5, all nodes blue. Cry (first) and River alternately move a token
//   along an edge; Cry wins when it reaches a node with no out-edges, River when it reaches a red
//   node (red sinks go to River). q <= 2e5 queries: paint u red, or ask whether Cry wins from u.
//   Multitest, sums of n, m, q <= 2e5.
// Solution: a node's outcome is the pair (Cry wins moving first, Cry wins moving second), one of
//   four classes, decided by how many out-neighbours fall in each class (`freq`); compute them in
//   reverse topological order. Painting red only turns win bits from true to false, so each node
//   changes class at most twice; `propagate` re-evaluates the in-neighbours of a changed node and
//   recurses, O(n + m + q) in total.

enum Outcome: BothCry | FirstWins | SecondWins | BothRiver

@!multitest
let (n, m, q): (1..=200_000, 0..=200_000, 1..=200_000) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let g = Graph.directed(edges)
let rev = g.transpose()

var state: [V]Outcome = Outcome.BothCry
var freq: [V, Outcome]Int = 0
var is_red: [V]Bool = false

fn eval(v: V) -> Outcome:
    if is_red[v]: return Outcome.BothRiver
    if g.deg(v) == 0: return Outcome.BothCry
    let has_c = freq[v, Outcome.BothCry] > 0
    let has_r = freq[v, Outcome.BothRiver] > 0
    let has_s = freq[v, Outcome.SecondWins] > 0
    if has_s or (has_c and has_r): return Outcome.FirstWins
    if has_r: return Outcome.BothRiver
    if has_c: return Outcome.BothCry
    Outcome.SecondWins

for v in peel(rev):
    state[v] = eval(v)
    for (u, _) in rev.out(v): freq[u, state[v]] += 1

fn propagate(v: V):
    let old_s = state[v]
    let new_s = eval(v)
    if new_s == old_s: return
    state[v] = new_s
    for (u, _) in rev.out(v):
        freq[u, old_s] -= 1
        freq[u, new_s] += 1
        propagate(u)

repeat q:
    let (op, v): (1..=2, V) = read()
    if op == 1:
        is_red[v] = true
        propagate(v)
    else:
        let cry_wins = state[v] == Outcome.BothCry or state[v] == Outcome.FirstWins
        print(yes(cry_wins))
