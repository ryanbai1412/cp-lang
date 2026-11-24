// Source: CCC 2022 S2 (Good Groups), https://dmoj.ca/problem/ccc22s2
// Problem: X <= 1e5 pairs of names that must share a group, Y <= 1e5 pairs that must not, then G
//   <= 1e5 groups of three names. Print how many of the X + Y constraints are violated.
// Solution: map each name to its group (Map<Str, Group>), then count the must-share pairs in
//   different groups and the must-not pairs in one group. O((X + Y + G) log G).
// original: corpus/submissions/4371343.cpp
let x: 0..=1e5 = read()
let together: [x](a: Str, b: Str) = read()
let y: 0..=1e5 = read()
let apart: [y](a: Str, b: Str) = read()
let g: 1..=1e5 = read()
domain Group = 0..g
let groups: [Group](a: Str, b: Str, c: Str) = read()

var group_of = Map<Str, Group>()
for k in Group:
    group_of[groups[k].a] = k
    group_of[groups[k].b] = k
    group_of[groups[k].c] = k
fn same(p: (a: Str, b: Str)) -> Bool = group_of[p.a] == group_of[p.b]
print(together.count(p => not same(p)) + apart.count(same))
