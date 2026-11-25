// expect: E0601
// rule: diagnostics E0601
// One place passed twice as mut.
fn two(mut a: List<Int>, mut b: List<Int>):
    a.push(1)
    b.push(2)
var xs = [0]
two(mut xs, mut xs)  // <- here
print(xs.len)
