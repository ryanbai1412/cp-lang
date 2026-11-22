// expect: ok, output "YES"
use std.strings.{SuffixArray}
var x = 12345
var w = List<Int>()
for i in 0..300:
    x = (x * 1103515245 + 12345) % 2147483648
    w.push(x / 65536 % 3)
let sa = SuffixArray(w)
var ok = true
for r in 1..w.len:
    let a = sa.sa[r - 1]
    let b = sa.sa[r]
    var k = 0
    while a + k < w.len and b + k < w.len and w[a + k] == w[b + k]: k += 1
    if k != sa.lcp[r - 1]: ok = false
    if not (a + k == w.len or (b + k < w.len and w[a + k] < w[b + k])): ok = false
for i in 0..40:
    for j in 0..40:
        var k = 0
        while i + k < w.len and j + k < w.len and w[i + k] == w[j + k]: k += 1
        if sa.lcp_between(i, j) != k: ok = false
print(ok)
