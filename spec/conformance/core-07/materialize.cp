// expect: ok, output "4 3 2"
// rule: 07.2.4 07.2.11
let a = [x * x for x in 0..4 if x % 2 == 0]
let b = List(0..3)
let c = (0..5).filter(x => x > 2).collect()
print(a.sum(), b.len, c.len)
