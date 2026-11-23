// Source: Mock CCC '22 Contest 1 S1 (Big Mattress Tournament), https://dmoj.ca/problem/nccc9s1
// Problem: a 1x1, b 1x2 and c L tiles (a 2x2 square minus its top-right cell), a, b, c <= 1e8.
//   Print YES if all of them, unrotated, tile a 2 x m rectangle for some m. T <= 1e4 tests.
// Solution: fill left to right. An L needs both rows level and leaves the bottom row one cell
//   ahead; 1x2s shift a row by 2, so only a 1x1 fixes an odd offset: each L needs its own 1x1
//   (a >= c). Without Ls, an odd b leaves the rows 2 apart, needing two 1x1s (a >= 2) unless
//   b is even. The area a + 2b + 3c must be even too. O(1) per test.
// original: corpus/submissions/4051682.cpp
@!multitest
let a: 0..=1_000_000_000 = read()
let b: 0..=1_000_000_000 = read()
let c: 0..=1_000_000_000 = read()
let ok = (a + 2 * b + 3 * c) % 2 == 0 and (if c > 0: a >= c else: b % 2 == 0 or a >= 2)
if ok: print("YES")
else: print("NO")
