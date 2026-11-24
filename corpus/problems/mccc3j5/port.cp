// Source: Mock CCC '22 Contest 1 J5 (New Year's Restrictions), https://dmoj.ca/problem/mccc3j5
// Problem: N <= 18 resolutions and M distinct pairs of them that cannot both be kept. Print the
//   most resolutions that can be kept (a maximum independent set).
// Solution: try all 2^N subsets (`Subset<V>`), keep those holding no restricted pair, and print
//   the largest popcount. O(2^N M).
// Bounds: n: 1..=18; m: 0..=153; restricted.u, restricted.v: 1..=n
// original: corpus/submissions/4223449.cpp
// Bounds: n: 1..=18; m: 0..=153; restricted.u, restricted.v: 1..=n
let (n, m): (1..=18, 0..=153) = read()
domain V = 1..=n
let restricted: [m](u: V, v: V) = read()
print(max(keep.len for keep in Subset<V> if all(not (e.u in keep and e.v in keep) for e in restricted))!)
