// expect: ok, output "2 3"
// rule: 07.12.7
var a = [1, 2, 3, 4]
a.rotate_left(1)
print(a[0], [1, 2, 3].rotated_right(1)[0])
