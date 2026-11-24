// Source: CEOI '19 Practice P2 (Separator), https://dmoj.ca/problem/ceoi19pp2
// Problem: append n <= 1e6 distinct values to an initially empty sequence, online: the i-th is
//   a_i = (b_i + s_{i-1}) mod 1e9, given b_i < 1e9. After each append print s_i, the number of
//   separators: indices j with every earlier element smaller and every later one larger.
// Solution: a separator is a prefix maximum that no later element undercuts. Keep the current
//   separators in an ordered set: a new prefix maximum joins it, then every kept value larger
//   than the new element is removed from the top. Each value enters and leaves once, so the
//   whole run is O(n log n); s_i is the set's size.
// original: corpus/submissions/5343197.cpp
let q: 1..=1e6 = read()
var good = Set<Int>()
var mx = -1
var last = 0
for _ in 0..q:
    let raw: 0..=1e9 = read()
    let x = (raw + last) % 1e9
    if x > mx:
        mx = x
        good.insert(x)
    while good.len > 0 and good.max()! > x: good.remove(good.max()!)
    last = good.len
    print(last)
