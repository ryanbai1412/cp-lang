// Source: CCO '18 P2 (Wrong Answer), https://dmoj.ca/problem/cco18p2
// Problem: output only. Print N <= 100 and costs A[i][j] in 1..100 (i < j) for a game where
//   two characters start at level 1 and must visit every other level exactly once, moving
//   only up; X is the given greedy's cost, Y the optimum; score ceil(min(25, X / 4Y)).
// Solution: all costs 100 except 1s on two interleaved chains, 0-1-2-5-6-9-10-... and
//   0-3-4-7-8-11-..., which cover every level, so Y = N - 1 = 99. Ties send the greedy onto
//   the wrong chains early, after which it pays 100 on almost every move: X = 9702.
// original: corpus/submissions/4801864.cpp

const N = 100
domain V = 0..N
var a: [V, V]Int = 100
a[V(0), V(1)] = 1
a[V(0), V(2)] = 1
a[V(0), V(3)] = 1
a[V(1), V(2)] = 1
var i = 2
while i + 3 < N:
    a[V(i), V(i + 3)] = 1
    a[V(i + 1), V(i + 2)] = 1
    i += 2

print(N)
for r in 0..N - 1:
    var row: List<Int> = []
    for c in r + 1..N:
        row.push(a[V(r), V(c)])
    print(row)
