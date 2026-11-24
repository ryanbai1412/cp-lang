// Source: Mock CCC '21 (NCCC 8) S3, Lexicographically Smallest Permutation Subsequence,
//   https://dmoj.ca/problem/nccc8s3
// limits: 100 ms, 1024 MB
// Problem: a list of N <= 2e5 integers in 1..K (K <= N) holds each of 1..K at least once;
//   print the lexicographically smallest permutation of 1..K that is a subsequence of it.
// Solution: greedy stack: scan left to right and skip values already kept; before pushing x,
//   pop every larger top that occurs again later (last[] holds each value's last position),
//   since it can be picked up after x. What stays on the stack is the answer. O(N).
// original: corpus/submissions/7011313.cpp
let (n, k) = read<(1..=200_000, 1..=200_000)>()
domain V = 1..=k
let a = read<[n]V>()
var last: [V]Int = 0
for (i, x) in a.items:
    last[x] = i
var kept: List<V> = []
var chosen: [V]Bool = false
for (i, x) in a.items:
    if chosen[x]:
        continue
    while let top = kept.last, top > x and last[top] > i:
        chosen[kept.pop()!] = false
    kept.push(x)
    chosen[x] = true
print(kept)
