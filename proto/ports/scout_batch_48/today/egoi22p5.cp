// EGOI '22 P5 (Data Centers): keep machine counts sorted descending; each
// service takes x from the top c, then one merge restores order.
// Runnable on proto/cpc today.
// original: corpus/submissions/5203693.cpp
let n: Int = read()
let q: Int = read()
let raw: [0..n]Int = read()
var a = sorted(raw, by: x => -x)
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
