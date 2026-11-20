let n: 1..=10 = read()
domain V = 1..=n
let v = V(2)
print(V.from_idx(9) == none, V.from_idx(0) == V.first)
