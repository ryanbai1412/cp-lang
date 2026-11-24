// Source: Yet Another Contest 7 P3 (No More Math Homework), https://dmoj.ca/problem/yac7p3
// Problem: a functional graph P on n <= 1e6 subjects; students i and j must share a group if
//   P^d(i) = P^d(j) for some d. Set one P_A = B to maximise the number of groups; print A B
//   and a group label for each student.
// Solution: a component with cycle length L allows exactly L groups, so the answer is the sum
//   of cycle lengths. Repoint the cycle predecessor of the deepest vertex's cycle root at
//   that vertex: the cycle gains its whole tail. Recompute the cycles with Functional and
//   label each vertex by (cycle position + depth) mod L plus the component's offset.
//   O(n log n).
// Bounds: n: 1..=1_000_000; par: 1..=n
// original: corpus/submissions/5705218.cpp
let n: 1..=1_000_000 = read()
domain V = 1..=n
var par: [V]V = read()
let f = Functional.new(par)
let deep = V.filter(v => not f.on_cycle(v)).max(by: v => (f.depth(v), v)) ?? V.first!
let moved = V.find(v => f.on_cycle(v) and par[v] == f.root_of(deep))!
par[moved] = deep
let g = Functional.new(par)
var idx: [V]Int = 0
var base: [V]Int = 0
var offset = 0
for cycle in g.cycles():
    for (i, v) in cycle.items:
        idx[v] = (cycle.len - i) % cycle.len
        base[v] = offset
    offset += cycle.len
fn label(v: V) -> Int:
    let r = g.root_of(v)
    base[r] + (idx[r] + g.depth(v)) % g.cycle_len(v) + 1
print(moved, deep)
print(V.map(label))
