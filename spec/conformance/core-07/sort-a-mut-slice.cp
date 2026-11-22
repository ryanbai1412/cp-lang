// expect: ok, output "4 2 3 1"
// rule: 07.1.5 07.2.12 07.3.6 07.11.1
var a = [4, 3, 2, 1]
a[1..3].sort()
print(a[0], a[1], a[2], a[3])
