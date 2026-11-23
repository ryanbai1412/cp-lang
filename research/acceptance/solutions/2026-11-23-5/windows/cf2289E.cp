// E. The Problem from Silach
// https://codeforces.com/contest/2289/problem/E
// x can beat y iff y > x - d, so x survives a match iff x < (weakest possible winner of the other side) + d.
// Keep each subtree's possible winners in a SortedSet, drop the too-weak ones from the top, merge small-to-large.
use std.bag.*
@!multitest
let (n, d): (2..=2e5, 1..=n) = read()
let m: [0..n - 1](1..=2 * n - 2, 1..=2 * n - 2) = read()
var parent: [0..2 * n]Int = 0
for (j, (p1, p2)) in m.items:
    parent[p1] = n + 1 + j
    parent[p2] = n + 1 + j
var dep: [0..2 * n]Int = 0
dep[2 * n - 1] = 1
for k in (n + 1..2 * n - 1).rev:
    dep[k] = dep[parent[k]] + 1
var win: [0..2 * n]SortedSet<Int> = SortedSet()
for x in 1..=n:
    win[x].add(x)
var lost_at: [0..n + 1]?Int = None
for (j, (l, r)) in m.items:
    let k = n + 1 + j
    let ml = win[l].last!
    let mr = win[r].last!
    while (win[l].last ?? 0) >= mr + d:
        lost_at[win[l].pop_last()!] = k
    while (win[r].last ?? 0) >= ml + d:
        lost_at[win[r].pop_last()!] = k
    let merged = win[l].take()
    win[k] = merged
    win[k].absorb(mut win[r])
print([dep[parent[x]] - (if let e = lost_at[x]: dep[e] - 1 else: 0) for x in 1..=n])
