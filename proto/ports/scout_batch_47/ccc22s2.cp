// CCC '22 S2 — Good Groups (DMOJ: ccc22s2)
// original: corpus/submissions/4371343.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Given X must-be-together name pairs, Y must-be-apart name pairs, and G
// 3-person groups partitioning all students, count violated constraints.
let x: 0..=1e5 = read()
let same: [0..x](Str, Str) = read()

let y: 0..=1e5 = read()
let diff: [0..y](Str, Str) = read()

let g: 1..=1e5 = read()
domain Group = 0..g
var group = HashMap<Str, Group>()
for id in Group:
    let (a, b, c): (Str, Str, Str) = read()
    group[a] = id
    group[b] = id
    group[c] = id

let bad_same = count(group[u] != group[v] for (u, v) in same)
let bad_diff = count(group[u] == group[v] for (u, v) in diff)
print(bad_same + bad_diff)
