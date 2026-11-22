// TLE '18 P5, longest increasing subsequence (buildable today-form)
// original: corpus/submissions/5876770.cpp
// The judge header's N and get_item become stdin; print_num becomes print.
let n: 0..=2000 = read()
let a: [0..n]Int = read()
var len: [0..n]Int = 1
var prev: [0..n]?Int = none
var end: ?Int = none
for i in 0..n:
    for j in 0..i:
        if a[j] < a[i] and len[j] >= len[i]:
            len[i] = len[j] + 1
            prev[i] = j
    if end == none or len[i] > len[end!]: end = i

// back along `prev` from the first index where the longest one ends
var path: List<Int> = []
var at = end
while at != none:
    path.push(a[at!])
    at = prev[at!]
print(path.len)
for t in 0..path.len: print(path[path.len - 1 - t])
