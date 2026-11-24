// Source: COCI 2021/2022 Contest 1 P3 (Logičari), https://dmoj.ca/problem/coci21c1p3
// limits: 1000 ms, 512 MB
// Problem: a connected graph with n <= 1e5 vertices and n edges. Colour the fewest vertices
//   blue so that every vertex has exactly one blue neighbour; print that count, or -1.
// Solution: the graph is a tree plus one edge (a, b), found with a DSU. Root the tree at a
//   and guess the colours of a and b; a tree DP over (vertex, its colour, parent's colour)
//   counts blues so each vertex gets exactly one blue among its parent, its children and,
//   for a or b, the other end of the extra edge. O(n) times 4 guesses.
// original: corpus/submissions/4080960.cpp
// Bounds: u, v: 1..=n

let n: Int = read()
domain V = 1..=n
var joined = Dsu<V>.new()
var g: [V]List<V> = []
var a = V.first!
var b = V.first!
for _ in 0..n:
    let u: V = read()
    let v: V = read()
    if joined.unite(u, v):
        g[u].push(v)
        g[v].push(u)
    else:
        a = u
        b = v

var kids: [V]List<V> = []
var seen: [V]Bool = false
var order: List<V> = [a]
seen[a] = true
var head = 0
while head < order.len:
    let u = order[head]
    head += 1
    for v in g[u]:
        if not seen[v]:
            seen[v] = true
            kids[u].push(v)
            order.push(v)

fn plus(x: ?Int, y: ?Int) -> ?Int:
    if x == None or y == None: return None
    x! + y!

@memo
fn blues(u: V, on: Bool, par_on: Bool, a_on: Bool, b_on: Bool) -> ?Int:
    if (u == a and on != a_on) or (u == b and on != b_on): return None
    let across = (u == a and b_on) or (u == b and a_on)
    if par_on and across: return None
    var zero: ?Int = Int(on)
    var one: ?Int = None
    for v in kids[u]:
        let off = blues(v, false, on, a_on, b_on)
        let lit = blues(v, true, on, a_on, b_on)
        one = min(plus(one, off), plus(zero, lit))
        zero = plus(zero, off)
    if par_on or across: zero else: one

var best: ?Int = None
for a_on in Bool:
    for b_on in Bool: best = min(best, blues(a, a_on, false, a_on, b_on))
print(best ?? -1)
