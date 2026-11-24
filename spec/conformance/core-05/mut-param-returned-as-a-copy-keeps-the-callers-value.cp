// expect: ok, output "1 1\n2 2"
// rule: 05.3.3
// A fn that returns a copy of its `mut` param leaves the caller's place
// holding the value too.
fn f(mut xs: List<Int>) -> List<Int>:
    xs.push(1)
    xs.clone()
var a: List<Int> = []
let b = f(mut a)
print(a.len, b.len)
let c = f(mut a)
print(a.len, c.len)
