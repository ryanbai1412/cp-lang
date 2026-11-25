// expect: E0601
// Lend global `a` mut; callee also reads `a` by name: error.
var a: List<Int> = [1]
fn fill(mut dst: List<Int>):
    for x in a: dst.push(x)
fill(mut a)  // <- here
print(a.len)
