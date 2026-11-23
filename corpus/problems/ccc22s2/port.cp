// Source: CCC 2022 S2 (Good Groups), https://dmoj.ca/problem/ccc22s2
// Problem: X <= 1e5 pairs of names that must share a group, Y <= 1e5 pairs that must not, then G
//   <= 1e5 groups of three names. Print how many of the X + Y constraints are violated.
// Solution: map each name to its group index (Map<Str, Int>), then count the must-share pairs in
//   different groups and the must-not pairs in one group. O((X + Y + G) log G).
// original: corpus/submissions/4371343.cpp
let x: 0..=100_000 = read()
var same_u: List<Str> = []
var same_v: List<Str> = []
for _ in 0..x:
    let u: Str = read()
    let v: Str = read()
    same_u.push(u)
    same_v.push(v)

let y: 0..=100_000 = read()
var diff_u: List<Str> = []
var diff_v: List<Str> = []
for _ in 0..y:
    let u: Str = read()
    let v: Str = read()
    diff_u.push(u)
    diff_v.push(v)

let g: 1..=100_000 = read()
var grp = Map<Str, Int>()
for id in 0..g:
    let a: Str = read()
    let b: Str = read()
    let c: Str = read()
    grp[a] = id
    grp[b] = id
    grp[c] = id

var violations = 0
for i in 0..x:
    if grp[same_u[i]] != grp[same_v[i]]: violations += 1
for i in 0..y:
    if grp[diff_u[i]] == grp[diff_v[i]]: violations += 1
print(violations)
