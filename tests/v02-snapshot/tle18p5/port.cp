// Source: TLE 2018-19 P5 (Pure Functional Programming), https://dmoj.ca/problem/tle18p5
// Problem: a sequence of N <= 1000 integers in [0, 1e9] (the judge exposes N and get_item,
//   and C++ code may not mutate). Print the length of a longest strictly increasing
//   subsequence, then its elements one per line.
// Solution: O(N^2) DP: len[i] is the longest increasing subsequence ending at i, prev[i]
//   the first j < i that gives it. Walk prev back from the first i with the largest len and
//   print the values reversed.
// original: corpus/submissions/5876770.cpp
// The judge header's N and get_item become stdin; print_num becomes print.
let n: 0..=2000 = read()
let a: [0..n]Int = read()
var len: [0..n]Int = 1
var prev: [0..n]?Int = None
var end: ?Int = None
for i in 0..n:
    for j in 0..i:
        if a[j] < a[i] and len[j] >= len[i]:
            len[i] = len[j] + 1
            prev[i] = j
    if end == None or len[i] > len[end!]: end = i

// back along `prev` from the first index where the longest one ends
var path: List<Int> = []
var at = end
while at != None:
    path.push(a[at!])
    at = prev[at!]
print(path.len)
for t in 0..path.len: print(path[path.len - 1 - t])
