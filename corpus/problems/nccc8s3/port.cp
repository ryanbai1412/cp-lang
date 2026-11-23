// Source: Mock CCC '21 (NCCC 8) S3, Lexicographically Smallest Permutation Subsequence,
//   https://dmoj.ca/problem/nccc8s3
// Problem: a list of N <= 2e5 integers in 1..K (K <= N) holds each of 1..K at least once;
//   print the lexicographically smallest permutation of 1..K that is a subsequence of it.
// Solution: greedy stack: scan left to right and skip values already kept; before pushing x,
//   pop every larger top that occurs again later (last[] holds each value's last position),
//   since it can be picked up after x. What stays on the stack is the answer. O(N).
// original: corpus/submissions/7011313.cpp
let n: 1..=2e5 = read()
let k: 1..=n = read()
domain I = 1..=n
domain V = 1..=k
let a: [I]V = read()

var last: [V]I = I.first!  // where each number occurs for the last time
for i in I: last[a[i]] = i

// Keep a stack of the numbers chosen so far. A new number pushes out every
// larger one on top of it that occurs again later.
var kept: List<V> = []
var chosen: [V]Bool = false
for i in I:
    let x = a[i]
    if chosen[x]: continue
    while let top = kept.last and top > x and last[top] > i:
        chosen[kept.pop()!] = false
    kept.push(x)
    chosen[x] = true
print(kept)
