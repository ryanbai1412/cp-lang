var m: HashMap<Int, Int> = []
m[3] max= -5
m[5] min= 5
m[6] += 2
m[4] gcd= 6
m[4] gcd= 4
m[7] *= 3
m[7] *= 3
m[5] min= 2
m[3] max= -9
m[8] -= 3
m[9] lcm= 4
m[9] lcm= 6
print(m.get(3) ?? 7, m.get(5) ?? 7, m.get(6) ?? 7, m.get(4) ?? 7, m.get(7) ?? 7, m.get(8) ?? 7, m.get(9) ?? 7)
domain D = 0..5
var dm: Map<D, Int> = []
dm[D(2)] min= 4
dm[D(2)] min= 9
dm[D(3)] *= 5
print(dm.get(D(2)) ?? -1, dm.get(D(3)) ?? -1, dm.len)
var om: Map<Int, ?Int> = []
om[1] min= 3
om[1] min= 2
print(om[1] ?? -1)
