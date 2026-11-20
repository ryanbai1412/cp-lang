// error: `a[l]` is changed on the right side of an assignment to `a[l]`
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
fn inc(mut x: Int) -> Int:
    x += 1
    return x
a[l] += inc(mut a[l])
print(a)
