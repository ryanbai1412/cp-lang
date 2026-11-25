// expect: ok, output "4 3"
// (spec) R05.10.1
// A closure over a `mut` parameter writes the caller's variable.
fn fill(mut xs: List<Int>, n: Int):
    let add = (x: Int) => xs.push(x)
    for i in 0..n: add(i)
var a: List<Int> = []
fill(mut a, 4)
print(a.len, a[3])
