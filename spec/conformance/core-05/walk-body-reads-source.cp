// expect: ok, output "3"
// rule: 05.9.1
fn each(xs: List<Int>) -> yields Int:
    for x in xs: yield x
var a = [1, 2]
var s = 0
for x in each(a):
    s += x * a[0]
print(s)
