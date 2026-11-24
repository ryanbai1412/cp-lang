// expect: ok, output "6 3 2 1"
// rule: 07.12.6
// `prev_permutation` steps down through every order, then wraps to the greatest one.
var a = [3, 2, 1]
var n = 1
while a.prev_permutation(): n += 1
print(n, a[0], a[1], a[2])
