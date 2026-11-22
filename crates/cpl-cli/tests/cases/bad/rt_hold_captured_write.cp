// error: `clobber` can't be called here: it changes `a`, and `a` is lent to `part`
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
fn clobber():
    a[V(1)] = 99
let part = a[l..=r]
clobber()
print(sum(part))
