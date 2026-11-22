// expect: ok, output "1 2 3 4\n4 1 2 3\n2 3 4 1"
var a = [1, 2, 3, 4]
a.rotate_left(4)
print(a)
a.rotate_left(-1)
print(a)
a.rotate_left(6)
print(a)
