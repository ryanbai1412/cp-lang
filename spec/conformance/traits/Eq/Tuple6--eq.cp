// expect: ok, output "YES NO"
let a = (1, 2, 3, 4, 5, 6)
let b = (1, 2, 3, 4, 5, 0)
print(yes(a == a), yes(a == b))
