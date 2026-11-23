// expect: ok, output "123 12"
// rule: 07.9.2
let k = [2, 3, 4]
print([1, 2, 3].fold((s, x) => s * 10 + x, init: 0), k.fold(lcm, init: 1))
