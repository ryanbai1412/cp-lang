// CEOI '19 Practice P1: count (i < j, k < l) with a[j] - a[i] == b[l] - b[k].
// original: corpus/submissions/5343174.cpp
(n, m): (1..=1500, 1..=1500) = read()
let a: [0..n]Int = read()
let b: [0..m]Int = read()
var gaps = HashMap<Int, Int>()
for j in 0..n:
    for i in 0..j: gaps[a[j] - a[i]] += 1
var ans = 0
for j in 0..m:
    for i in 0..j: ans += gaps.get(b[j] - b[i]) ?? 0
print(ans)
