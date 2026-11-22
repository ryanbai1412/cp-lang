// expect: E0302 at 3
// rule: 04a.6.1 04b.3.1
fn f(xs: List<Int>) -> Int = xs.nth(0)
let s = [1, 2, 3, 4]
print(f(s[1..3]))
