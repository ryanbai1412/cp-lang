// expect: ok, output "0 3\n1 4\n2 6"
domain V = 0..3
let a: [V]Int = [3, 1, 2]
for (v, t) in a.scan(+).items(): print(v, t)
