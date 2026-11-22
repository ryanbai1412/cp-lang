// Batch #11: an end of a domain orders a step. Inside `if i > D.first:`
// `i - 1` is a `D`, and inside `if i < D.last:` so is `i + 1`.
let n: Int = read()
domain D = 0..n
let a: [D]Int = read()
fn walk(l: D, r: D) -> Int:
    if l < r: return a[l] + walk(l.next!, r)
    a[r]
fn left(i: D) -> Int:
    if let p = i.prev: return a[p]
    -1
fn right(i: D) -> Int:
    if let nx = i.next: return a[nx]
    -1
var s = 0
for i in D:
    if let p = i.prev: s += a[p]
    if let nx = i.next: s += a[nx]
    if let p = i.prev, let nx = i.next: s += a[p] * a[nx]
print(walk(D.first!, D.last!), s)
print([left(i) for i in D], [right(i) for i in D])
