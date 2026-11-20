// error: `a[l]` can't be passed to `go`: `go` also changes `a` directly
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
fn go(mut x: Int, i: V):
    a[i] += x
    x += 1
    if i != V.last:
        go(mut x, i + 1)
go(mut a[l], l)
print(a)
