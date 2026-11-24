// Source: CCC 1996 S3 (Pattern Generator), https://dmoj.ca/problem/ccc96s3
// Problem: for each of several pairs n, k (n <= 30, k < 8), print "The bit patterns are" and then
//   all length-n bit patterns with k ones in descending binary order; blank line between cases.
// Solution: a pattern is the set of positions of its ones; `combinations` yields those sets in
//   lexicographic order, which puts ones as far left as possible first, so the patterns come out
//   in descending order. O(C(n, k) * n) per case.
// original: corpus/submissions/5515585.cpp
@!multitest
let n: 1..=30 = read()
let k: 0..=min(n, 7) = read()
print("The bit patterns are")
for ones in (0..n).combinations(k):
    print(Str([if ones.contains(i): '1' else: '0' for i in 0..n]))
print()
