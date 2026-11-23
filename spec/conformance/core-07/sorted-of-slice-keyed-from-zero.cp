// expect: ok, output "1 3"
// rule: 07.11.1
let a = [9, 1, 5, 3]
let s = a[1..4].sorted()
print(s[0], s[1])
