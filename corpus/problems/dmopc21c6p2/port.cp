// Source: DMOPC '21 Contest 6 P2 (Gacha Luck), https://dmoj.ca/problem/dmopc21c6p2
// Problem: a 0/1 string of length n <= 1e6 and k <= n. A substring scores its number of 0s,
//   halved once per 1 in it, rounded up. Print the largest total score of k disjoint substrings.
// Solution: a substring with a 1 never beats its best run of 0s (ceil((a + b) / 2) <= max(a, b)),
//   so the answer is the sum of the k longest maximal runs of 0s: split on the 1s and sort the
//   run lengths descending. O(n log n).
// original: corpus/submissions/4342045.cpp
// Lengths of the maximal runs of '0'; a trailing sentinel closes the last run.

let n: 1..=1e6 = read()
let k: 1..=n = read()
let s: Str = read()

var runs = [w.len for w in s.split('1') if w.len > 0]
runs.sort(by: x => desc(x))
print(runs[0..min(k, runs.len)].sum())
