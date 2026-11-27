// expect: ok, output "YES NO"
let a = (1, 2, 3, 4, 5, 6, 7, 8)
let b = (1, 2, 3, 4, 5, 6, 7, 0)
print(yes(b < a), yes(a < b))
