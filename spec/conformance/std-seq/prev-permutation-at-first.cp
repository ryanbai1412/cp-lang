// expect: ok, output "NO YES"
var a = [1, 2, 3]
let moved = a.prev_permutation()
print(yes(moved), yes(a == [3, 2, 1]))
