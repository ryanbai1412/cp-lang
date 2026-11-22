// expect: ok, output "6"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
let a: [D]Int = [D] x => 1
print(a.sum())
