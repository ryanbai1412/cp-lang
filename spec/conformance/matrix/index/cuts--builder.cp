// expect: ok, output "4"
domain V = 0..3
let a: [V.cuts]Int = [V.cuts] x => 1
print(a.sum())
