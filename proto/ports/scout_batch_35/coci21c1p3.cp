// COCI 2021/22 #1 Logičari: a connected graph with n vertices and n edges.
// Colour some vertices blue so that every vertex has exactly one blue
// neighbour; print the fewest blue vertices, or -1.
//
// Drop one cycle edge (a, b) and root the tree that is left at a. Guess the
// colours of a and b; everything else is a tree DP where a vertex knows its
// own colour and its parent's.
// original: corpus/submissions/4080960.cpp
let n: 3..=1e5 = read()
domain V = 1..=n
let edges: [0..n](V, V) = read()

var joined = Dsu<V>()
let (a, b) = first(e for e in edges if not joined.unite(e.0, e.1))!
let tree = Rooted<V>(Graph.undirected(e for e in edges if e != (a, b)), a)

fn plus(x: ?Int, y: ?Int) -> ?Int: x? + y?

// fewest blue vertices in u's subtree when u is blue iff `on`, its parent iff `par_on`
@memo
fn blues(u: V, on: Bool, par_on: Bool, a_on: Bool, b_on: Bool) -> ?Int:
    if (u == a and on != a_on) or (u == b and on != b_on): return none
    let across = (u == a and b_on) or (u == b and a_on)    // blue over the dropped edge
    if par_on and across: return none
    var zero: ?Int = if on: 1 else: 0                       // no blue child so far
    var one: ?Int = none                                    // exactly one
    for v in tree.children(u):
        let (off, lit) = (blues(v, false, on, a_on, b_on), blues(v, true, on, a_on, b_on))
        one = min(plus(one, off), plus(zero, lit))
        zero = plus(zero, off)
    if par_on or across: zero else: one

var best: ?Int = none
for a_on in Bool:
    for b_on in Bool: best min= blues(a, a_on, false, a_on, b_on)
print(best, none: -1)
