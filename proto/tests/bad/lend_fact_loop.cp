// error: only when `i != j`
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
var i = l
assert i != r
while a[l] < 100:
    swap(mut a[i], mut a[r])
    i = r
    a[l] += 50
