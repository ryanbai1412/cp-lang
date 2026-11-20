// error: `outer` can't be called here: it changes `a`, and `a` is lent to `part`
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
fn clobber():
    a[V(1)] = 99
fn outer():
    clobber()
let part = a[l..=r]
outer()
print(sum(part))
