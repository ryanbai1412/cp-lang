// expect: ok, output "2"
// rule: 04b.3.2 04b.1.6
fn f(xs: List<Int>) -> Int = xs[0]
let s = [1, 2, 3, 4]
print(f(List(s[1..3])))
