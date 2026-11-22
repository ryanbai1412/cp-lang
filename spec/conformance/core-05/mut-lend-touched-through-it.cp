// expect: ok, output "2"
// rule: 05.7.7
fn bump(mut xs: List<Int>): xs[0] += 1
var a = [1]
bump(mut a)
print(a[0])
