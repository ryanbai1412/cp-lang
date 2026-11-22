// expect: ok, output "5 0"
// rule: 05.6.3
var a = [5, 6]
a.at(1) = 0
print(a[0], a[1])
