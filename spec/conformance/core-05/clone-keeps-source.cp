// expect: ok, output "3 2"
// rule: 05.2.2
var xs = List(1, 2)
var ys = xs.clone()
ys.push(3)
print(ys.len, xs.len)
