// expect: ok, output "4"
domain D = 0..4
let a: [D]Int = [D] x => 1
print(a.sum())
