// expect: ok, output "2 8"
// rule: 05.3.2
var x = 1
x = 2
var xs = [1]
xs.push(5)
xs[0] = 3
print(x, xs.sum())
