// Source: NOIP 2001 Senior P2 (Splitting a Number), https://dmoj.ca/problem/noip01p2
// limits: 1000 ms, 125 MB
// Problem: 6 < n <= 200, 2 <= k <= 6. Count the ways to split n into exactly k positive parts,
//   order ignored.
// Solution: either some part is 1 (drop it: ways(n - 1, k - 1)) or every part is >= 2 (take 1
//   off each: ways(n - k, k)). Memoised over (total, parts): O(nk).
// original: corpus/submissions/5086680.cpp
let n: 1..=200 = read()
let k: 1..=6 = read()

@memo
fn ways(total: 0..=200, parts: 0..=6) -> Int:
    if parts == 0: return Int(total == 0)
    if total < parts: return 0
    ways(total - 1, parts - 1) + ways(total - parts, parts)
print(ways(n, k))
