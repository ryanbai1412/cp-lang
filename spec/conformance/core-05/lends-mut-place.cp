// expect: ok, output "7 5"
// rule: 05.6.3 05.6.4
fn head(mut xs: List<Int>) -> lends mut Int = xs[0]
var a = [1, 2]
head(mut a) = 7
a[1] = 5
print(a[0], a[1])
