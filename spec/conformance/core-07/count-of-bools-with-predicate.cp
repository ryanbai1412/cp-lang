// expect: ok, output "2 4"
// rule: 07.9.4
// The trues are `count(x => x)`; every element is `len`.
let b = [true, false, true, false]
print(b.count(x => x), b.len)
