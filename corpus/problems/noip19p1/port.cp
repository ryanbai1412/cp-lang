// Source: NOIP 2019 P1 (CSP-S 2019, Gray Code), https://dmoj.ca/problem/noip19p1
// Problem: the n-bit Gray code (n <= 64) is 0, 1 for n = 1; for n + 1 bits it is the n-bit list
//   prefixed by 0, then the n-bit list reversed and prefixed by 1. Print string number k,
//   0 <= k < 2^n.
// Solution: string number k is k ^ (k >> 1): print bit i of k xor bit i + 1, most significant
//   first. O(n).
// original: corpus/submissions/5127723.cpp
// Bounds: n: 1..=64
let n: 1..=64 = read()
let k: U64 = read()
let gray = k ^ (k >> 1)
print([if (gray >> i) & U64(1) == U64(1): '1' else: '0' for i in (0..n).rev])
