// Source: DMOPC '21 Contest 6 P2 (Gacha Luck), https://dmoj.ca/problem/dmopc21c6p2
// Problem: a 0/1 string of length n <= 1e6 and k <= n. A substring scores its number of 0s,
//   halved once per 1 in it, rounded up. Print the largest total score of k disjoint substrings.
// Solution: a substring with a 1 never beats its best run of 0s (ceil((a + b) / 2) <= max(a, b)),
//   so the answer is the sum of the k longest maximal runs of 0s: collect the run lengths and
//   sort them descending. O(n log n).
// original: corpus/submissions/4342045.cpp

let (n, k): (Int, Int) = read()
let s: Str = read()

// Lengths of the maximal runs of '0'; a trailing sentinel closes the last run.
var zero_runs: List<Int> = []
var run = 0
for c in s + "1":
    if c == '0':
        run += 1
    elif run > 0:
        zero_runs.push(run)
        run = 0

let longest = zero_runs.sorted(by: x => -x)
var ans = 0
for i in 0..min(k, longest.len): ans += longest[i]
print(ans)
