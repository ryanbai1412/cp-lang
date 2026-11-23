let n: 1..=10 = read()
domain V = 1..=n
let v = V(2)
print(V.try_at(9) == None, V.try_at(0) == V.first)
