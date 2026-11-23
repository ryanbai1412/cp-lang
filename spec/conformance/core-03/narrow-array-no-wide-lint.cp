// expect: ok, output "7"
// rule: 03.1.8
var a: [1000000]0..=1e9 = [1000000].filled(0)
a[0] = 7
print(a[0])
