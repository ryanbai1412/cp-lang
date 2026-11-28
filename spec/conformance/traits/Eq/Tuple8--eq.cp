// expect: ok, output "YES NO"
// engines: next
let a = (1, 2, 3, 4, 5, 6, 7, 8)
let b = (1, 2, 3, 4, 5, 6, 7, 0)
print(yes(a == a), yes(a == b))
