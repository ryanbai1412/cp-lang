// Source: Mock CCC '22 Contest 1 J5 (New Year's Restrictions), https://dmoj.ca/problem/mccc3j5
// Problem: N <= 18 resolutions and M distinct pairs of them that cannot both be kept. Print the
//   most resolutions that can be kept (a maximum independent set).
// Solution: try all 2^N subsets (`Subset<V>`), keep those holding no restricted pair, and print
//   the largest popcount. O(2^N M).
// Written for CP Lang 0.2.2.
// original: corpus/submissions/4223449.cpp
let (n, m): (Int, Int) = read()
domain V = 1..=n
var edges: List<(V, V)> = []
for _ in 0..m:
    let (u, v): (V, V) = read()
    edges.push((u, v))

var ans = 0
for mask in Subset<V>:
    if all(not (u in mask and v in mask) for (u, v) in edges):
        ans max= popcount(mask.ord)
print(ans)
