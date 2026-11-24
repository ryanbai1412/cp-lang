// Source: TLE 2018-19 P5 (Pure Functional Programming), https://dmoj.ca/problem/tle18p5
// limits: 1000 ms, 256 MB
// Problem: a sequence of N <= 1000 integers in [0, 1e9] (the judge exposes N and get_item,
//   and C++ code may not mutate). Print the length of a longest strictly increasing
//   subsequence, then its elements one per line; any longest one is accepted.
// Solution: the stdlib's patience-sorting `lis` gives the positions of one longest strictly
//   increasing subsequence; print its length and the values there. O(N log N).
// original: corpus/submissions/5876565.cpp
// The judge header's N and get_item become stdin; print_num becomes print.
let n = read<1..=1000>()
let a = read<[n]0..=1e9>()
let best = a.lis()
print(best.len)
for i in best: print(a[i])
