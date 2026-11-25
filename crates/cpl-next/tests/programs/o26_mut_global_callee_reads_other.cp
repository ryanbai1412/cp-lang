// expect: ok, output "3"
// False-rejection probe: lend global `a` mut; callee reads global `b` by name.
var a: List<Int> = []
var b: List<Int> = [1, 2, 3]
fn fill(mut dst: List<Int>):
    for x in b: dst.push(x)
fill(mut a)
print(a.len)
