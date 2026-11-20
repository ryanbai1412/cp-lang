let n: 1..=10 = read()
domain V = 1..=n
let v = V(2)
print(V.K, V.first, V.last, v.to_idx, v + 1, v + 9, v - 5, V(4) - v, v.cyc(3))
