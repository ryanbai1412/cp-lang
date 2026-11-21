let a = [3, 9, 2, 9]
print(argmax(a) ?? -1, argmin(a) ?? -1)
print(argmax(a[i] for i in 0..4) ?? -1)
let b: [0..3]Int = [5, 1, 5]
let none_here: List<Int> = []
print(argmax(b) ?? -1, argmin(b) ?? -1, argmin(none_here) ?? -1)
