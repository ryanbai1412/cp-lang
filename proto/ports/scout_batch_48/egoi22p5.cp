// EGOI '22 P5 (Data Centers): keep machine counts sorted descending; each
// service takes x from the top c, after which a[..c] and a[c..] are each
// still sorted, so one two-pointer merge restores order.
// original: corpus/submissions/5203693.cpp
(n, q): (1..=1e5, 0..=5000) = read()
let raw: [0..n]Int = read()
var a = sorted(raw, by: x => -x)
for _ in 0..q:
    (x, c): (Int, 0..=n) = read()
    var merged: List<Int> = []
    var (i, j) = (0, c)
    while i < c or j < n:
        if j == n or (i < c and a[i] - x >= a[j]):
            merged.push(a[i] - x)
            i += 1
        else:
            merged.push(a[j])
            j += 1
    a = merged
print(a)
