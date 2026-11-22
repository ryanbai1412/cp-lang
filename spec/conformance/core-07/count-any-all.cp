// expect: ok, output "2 YES YES"
// rule: 07.9.4
let a = [1, 2, 3]
print(a.count(x => x > 1), yes(a.any(x => x > 2)), yes(a.all(x => x > 0)))
