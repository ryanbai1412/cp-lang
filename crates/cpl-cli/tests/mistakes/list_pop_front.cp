// here: 4
// no fix: a list has no O(1) front pop; Python's `a.pop(0)` habit as `pop_front`.
var a = [1, 2, 3]
let x = a.pop_front()
print(x)
