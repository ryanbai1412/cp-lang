// expect: ok, output "3 11 4 9 5"
var a = [3, 1, 4, 1, 5]
a[1..4][3] = 9
a[1..4][1] += 10
print(a)
