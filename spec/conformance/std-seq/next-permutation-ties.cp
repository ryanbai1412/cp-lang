// expect: ok, output "3"
var a = [1, 1, 2]
var n = 1
while a.next_permutation(): n += 1
print(n)
