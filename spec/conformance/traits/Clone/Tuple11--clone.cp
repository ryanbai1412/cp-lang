// expect: ok, output "x 2 3 4 5 6 7 8 9 10 11 x 2 3 4 5 6 7 8 9 10 11"
// engines: next
let a = ("x", 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
let b = a.clone()
print(a, b)
