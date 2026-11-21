// Batch #11: an end of a domain orders a step. Inside `if i > D.first:`
// `i - 1` is a `D`, and inside `if i < D.last:` so is `i + 1`.
let n: Int = read()
domain D = 0..n
let a: [D]Int = read()
fn walk(l: D, r: D) -> Int:
    if l < r: return a[l] + walk(l + 1, r)
    a[r]
fn left(i: D) -> Int:
    if i > D.first: return a[i - 1]
    -1
fn right(i: D) -> Int:
    if i == D.last: return -1
    a[i + 1]
var s = 0
for i in D:
    if i > D.first: s += a[i - 1]
    if i < D.last: s += a[i + 1]
    if D.first < i and i < D.last: s += a[i - 1] * a[i + 1]
print(walk(D.first, D.last), s)
print([left(i) for i in D], [right(i) for i in D])
