// error: Array.at lends disjoint parts only when `i != j`, and that is not known here
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
var i = V(1)
let r = V(2)
fn mv(mut x: Int, mut y: Int):
    y += x
    x = 0
fn step():
    i = V(2)
if i != r:
    step()
    mv(mut a[i], mut a[r])
print(a)
