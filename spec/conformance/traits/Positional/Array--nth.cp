// expect: ok, output "6"
// engines: next
domain V = 0..3
let a: [V]Int = [4, 5, 6]
print(a.nth(2))
