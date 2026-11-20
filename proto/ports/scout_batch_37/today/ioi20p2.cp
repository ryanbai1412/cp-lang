// IOI 2020 Day 1 P2, Connecting Supertrees (buildable today-form)
// original: corpus/submissions/4724578.cpp (grader function delivery)
// stdin here, as the IOI sample grader: n, then the n x n matrix p.
// stdout: 0, or 1 followed by the bridge matrix.
let n: 1..=1000 = read()
domain V = 0..n
let p: [V, V]0..=3 = read()

// Towers with p == 1 form trees; hang each tree off its first tower as a star.
var root: [V]V = V.first
var roots: List<V> = []
var placed: [V]Bool = false
for i in V:
    if placed[i]: continue
    roots.push(i)
    for j in V:
        if not placed[j] and p[i, j] == 1:
            placed[j] = true
            root[j] = i

// Trees whose roots have p == 2 sit on one cycle; ring[v] names that cycle.
var ring: [V]V = V.first
var bridge: [V, V]Bool = false
var ok = all(p[i, j] != 3 for i in V for j in V)
var joined: [V]Bool = false
for i in roots:
    if joined[i]: continue
    var cycle: List<V> = []
    for j in roots:
        if not joined[j] and (j == i or p[i, j] == 2):
            joined[j] = true
            ring[j] = i
            cycle.push(j)
    if cycle.len == 2: ok = false
    if cycle.len > 2:
        for k in 0..cycle.len:
            let (a, b) = (cycle[k], cycle[(k + 1) % cycle.len])
            bridge[a, b] = true
            bridge[b, a] = true
for v in V:
    if v != root[v]:
        bridge[v, root[v]] = true
        bridge[root[v], v] = true

// the layout determines every path count, so p must agree with it exactly
for i in V:
    for j in V:
        let want = if root[i] == root[j]: 1 elif ring[root[i]] == ring[root[j]]: 2 else: 0
        if p[i, j] != want: ok = false

if not ok:
    print(0)
else:
    print(1)
    for i in V: print([Int(bridge[i, j]) for j in V])
