// expect: ok, output "99 99 2 c 26\n5\na\nb\n3"
// rule: R04.2.24
// A `Char` domain's positions are its characters' codes: `L(c)` takes a `Char`, `Char(l)`
// gives it back, and a body declares one the same way.
domain L = 'a'..='z'
let l = L('c')
print(l, Int(l), l.ord, Char(l), L.count)
var a: [L]Int = 0
a[l] += 5
print(a[L('c')])
for x in L:
    if x.ord < 2: print(Char(x))
fn small() -> Int:
    domain M = 'x'..='z'
    M.count
print(small())
