// expect: ok, output "0\n1"
// rule: 05.7.2
struct Holder: xs: List<Int>
fn f(mut xs: List<Int>):
    let h = Holder(xs)
    print(h.xs.len)
    xs.push(1)
    print(xs.len)
var a: List<Int> = []
f(mut a)
