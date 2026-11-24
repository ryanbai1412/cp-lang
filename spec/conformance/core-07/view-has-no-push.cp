// expect: E0401
// rule: 07.2.11
var a = [1, 2, 3]
a.rev.push(4) // <- here
print(a.len)
