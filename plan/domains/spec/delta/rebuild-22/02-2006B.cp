// cf2006B Iris and the Tree: vertices are in DFS order, so path i -> i mod n + 1 uses
// edge x (x to its parent) exactly for i = x - 1 and i = x + size(x) - 1. Every edge lies
// on two paths; a path with an unknown edge left can take all of the unassigned weight.
@!multitest
let (n, w): (2..=2e5, 0..=1e12) = read()
domain V = 1..=n
var par: [V]?V = None
for v in V.drop(1): par[v] = Some(read<V>())
let t = Rooted.from_parents(par)
fn paths_of(x: V) -> List<V> = [V(x - 1), V(x + t.size(x) - 1)]

var unknown: [V]Int = 0           // unknown edges on path i
for x in V.drop(1):
    for p in paths_of(x): unknown[p] += 1
var open = n                      // paths with an unknown edge
var known = 0
var answers = List<Int>()
repeat n - 1:
    let (x, y): (V, 0..=w) = read()
    known += y
    for p in paths_of(x):
        unknown[p] -= 1
        if unknown[p] == 0: open -= 1
    answers.push(2 * known + (w - known) * open)
print(answers)
