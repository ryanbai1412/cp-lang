// expect: E0302 at 3
// rule: 07.5.11
fn f(xs: List<Int>) -> Int = xs.len
let a = [1, 2, 3]
print(f(a[1..]))
