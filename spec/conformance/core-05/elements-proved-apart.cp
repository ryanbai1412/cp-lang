// expect: ok, output "3 1"
// rule: 05.8.2
fn f(mut a: List<Int>, i: Int, j: Int):
    if i != j: swap(mut a[i], mut a[j])
var a = [1, 2, 3]
f(mut a, 0, 2)
print(a[0], a[2])
