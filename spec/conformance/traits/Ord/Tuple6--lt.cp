// expect: ok, output "YES NO"
// engines: next
let a = (1, 2, 3, 4, 5, 6)
let b = (1, 2, 3, 4, 5, 0)
print(yes(b < a), yes(a < b))
