// expect: ok, output "2 2 0"
// rule: 07.9.5
let a = [1, 2, 3]
print(a.first(x => x > 1)!, a.last(x => x < 3)!, a.first(x => x > 5) ?? 0)
