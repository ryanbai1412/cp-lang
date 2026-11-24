// Source: EGOI 2022 P5 (Data Centers), https://dmoj.ca/problem/egoi22p5
// Problem: n <= 1e5 data centres with at most 1e9 free machines each, and s <= 5000 services
//   in order: service i sorts the centres by free machines, descending, and takes m_i machines
//   from each of the top c_i (always enough). Print the final counts in descending order.
// Solution: keep the counts sorted descending. After taking m from the top c, both a[..c] - m
//   and a[c..] are still descending, so one two-pointer merge restores the order instead of
//   a sort. O(n log n + s n).
// Runnable on proto/cpc today.
// original: corpus/submissions/5203693.cpp
let n: Int = read()
let q: Int = read()
let raw: [0..n]Int = read()
var a = raw.sorted(by: x => -x)
for _ in 0..q:
    let x: Int = read()
    let c: Int = read()
    // a[..c] - x and a[c..] are both still descending: two-pointer merge
    var merged: List<Int> = []
    var i = 0
    var j = c
    while i < c or j < n:
        if j == n or (i < c and a[i] - x >= a[j]):
            merged.push(a[i] - x)
            i += 1
        else:
            merged.push(a[j])
            j += 1
    for k in 0..n: a[k] = merged[k]
print(a)
