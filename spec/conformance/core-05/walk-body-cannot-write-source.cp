// expect: E0602 at 5
// rule: 05.9.1 05.7.6
fn each(xs: List<Int>) -> yields Int:
    for x in xs: yield x
var a = [1, 2]
for x in each(a):
    a[0] = x
