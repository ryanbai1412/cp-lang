// Source: Yet Another Contest 6 P2 (No More Telemarketers), https://dmoj.ca/problem/yac6p2
// limits: 3000 ms, 512 MB
// Problem: two rooted trees s, t on N <= 1e6 workers (boss or -1). A move re-hangs a worker A
//   under any B that A does not manage. Print the fewest moves turning s into t and the moves
//   as pairs (A, B), or -1 if impossible.
// Solution: the root manages everyone, so it never moves: different roots give -1. Otherwise
//   each worker whose boss differs moves exactly once, to its t-boss, in a preorder of t: the
//   new boss's chain to the root is already final, so it cannot pass through the worker. O(N).
// original: corpus/submissions/5149548.cpp
let n = read<1..=1e6>()
domain V = 1..=n
let have = read<[V]?V>(none: -1)
let want = read<[V]?V>(none: -1)

fn root(boss: [V]?V) -> V = V.find(v => boss[v] == None)!

if root(have) != root(want): print(-1)
else:
    let moved = [v for v in Rooted.from_parents(want).at if have[v] != want[v]]
    print(moved.len)
    for v in moved: print(v, want[v]!)
