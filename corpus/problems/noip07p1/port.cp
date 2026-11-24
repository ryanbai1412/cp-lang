// Source: NOIP 2007 P1 (Counting Numbers), https://dmoj.ca/problem/noip07p1
// limits: 1000 ms, 128 MB
// Problem: n <= 200000 numbers, each at most 1.5e9, at most 10000 of them distinct. Print each
//   distinct value with how often it occurs, in increasing order of value.
// Solution: sort; each run of equal values is one output line, its value and its length.
//   O(n log n).
// original: corpus/submissions/5184805.cpp
let n: 1..=200000 = read()
let xs: [0..n]0..=1500000000 = read()
for run in xs.sorted().runs(): print(run.first!, run.len)
