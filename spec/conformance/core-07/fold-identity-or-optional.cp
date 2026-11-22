// expect: ok, output "6 312"
// rule: 07.9.2
let a = [3, 1, 2]
print(a.fold(+), a.fold((x, y) => x * 10 + y)!)
