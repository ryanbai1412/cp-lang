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

// The two cost-1 chains 0-1-2-5-6-9-10-... and 0-3-4-7-8-11-...
fn cheap(r: V, c: V) -> Bool:
    if r.ord < 2: return c.ord <= 3 and (r.ord, c.ord) != (1, 3)
    if r.ord % 2 == 0: c.ord == r.ord + 3
    else: c.ord == r.ord + 1 and r.ord + 2 < N

print(N)
for r in V.all.drop_last(1):
    print([if cheap(r, c): 1 else: 100 for c in V if c > r])
