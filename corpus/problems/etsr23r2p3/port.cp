// Source: ETSR 2023 Round 2 P3, https://dmoj.ca/problem/etsr23r2p3
// Problem: t triples of positive integers a, b, c (the statement is not mirrored here; the
//   bounds follow the accepted C++, whose `1 << (ctz + 1)` stays in `int` for values <= 1e9).
//   Print YES when the triple can be made equal, which the accepted solution decides as: both
//   gaps of the sorted triple are multiples of 2^(ctz(min) + 1).
// Solution: sort, take the step 2^(v + 1) with v the 2-adic valuation of the smallest, and
//   test both gaps. O(1) per case.
// original: corpus/submissions/5494453.cpp
@!multitest
let xs = read<[3]1..=1e9>().sorted()
let step = 1 << (ctz(xs[0]) + 1)
print(yes((xs[1] - xs[0]) % step == 0 and (xs[2] - xs[1]) % step == 0))
