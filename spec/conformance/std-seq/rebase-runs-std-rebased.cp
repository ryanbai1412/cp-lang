// expect: ok, output "6 8 3\n21\n23\n3 6\n0"
// rule: 04b.1.7
// `rebase` keys a slice from 0 by a fresh domain, and std's `Rebased` answers for it: `nth`,
// `len`, a `for`, `.items` and `List(..)`, inline and through a `let`.
let a = [5, 6, 7, 8]
let w = a[1..]
print(w.rebase.nth(0), w.rebase.nth(2), w.rebase.len)
var s = 0
for x in w.rebase: s += x
print(s)
var ks = 0
for (k, x) in w.rebase.items: ks += k.ord * x
print(ks)
let l = List(w.rebase)
print(l.len, l[0])
let t = w.rebase
var u = 0
for (k, x) in t.items: u += t[k] - x
print(u)
