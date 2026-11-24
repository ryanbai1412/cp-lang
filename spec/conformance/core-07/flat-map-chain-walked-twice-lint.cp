// expect: L0602
// rule: 07.1.3 07.2.9
let a = [1, 2, 3]
let c = a.flat_map(x => 0..x).map(x => x * 2)
print(c.sum())
print(c.sum()) // <- here
