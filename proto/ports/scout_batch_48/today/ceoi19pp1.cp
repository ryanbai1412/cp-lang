// CEOI '19 Practice P1 (Translation pairs): count (i < j, k < l) with
// a[j] - a[i] == b[l] - b[k]. Runnable on proto/cpc today.
// original: corpus/submissions/5343174.cpp
let n: Int = read()
let m: Int = read()
let a: [0..n]Int = read()
let b: [0..m]Int = read()
var gaps = Map<Int, Int>()
for j in 0..n:
    for i in 0..j: gaps[a[j] - a[i]] = (gaps.get(a[j] - a[i]) ?? 0) + 1
var ans = 0
for j in 0..m:
    for i in 0..j: ans += gaps.get(b[j] - b[i]) ?? 0
print(ans)
