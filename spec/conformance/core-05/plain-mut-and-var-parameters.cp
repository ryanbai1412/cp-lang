// expect: ok, output "2 3 2"
// rule: 05.2.3
// A plain parameter is lent, `mut` lends the caller's place, `var` owns its argument.
fn count(xs: List<Int>) -> Int = xs.len
fn push_one(mut xs: List<Int>):
    xs.push(1)
fn owned(var xs: List<Int>) -> Int:
    xs.push(2)
    xs.len
var a = [5]
push_one(mut a)
print(count(a), owned(a.clone()), a.len)
