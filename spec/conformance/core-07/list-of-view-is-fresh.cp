// expect: ok, output "9 3 2"
// rule: 07.5.8 07.4.4
var a = [1, 2, 3]
var l = List(a[1..])
a[1] = 9
l[0] += 1
print(a[1], l[0], l.len)
