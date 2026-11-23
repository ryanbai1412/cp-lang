// expect: ok, output "9 2 3\n9 2 3\n2 3 9\n2 3 9 0 0\n7 7 7 7 7"
// Acceptance for seq/U58 (insert, remove, swap, fill, reverse, rotate, resize), handbook/product/std/sequences-usecases.md.
var a = [1, 2, 3]
a.insert(1, 9)
a.remove(0)
print(a)
a.swap(0, 2)
a.reverse()
print(a)
a.rotate_left(1)
print(a)
a.resize(5, 0)
print(a)
a.fill(7)
print(a)
