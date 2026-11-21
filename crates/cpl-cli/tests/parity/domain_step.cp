let n: Int = read()
domain V = 1..=n
var w: [V]Int = 0
for v in V by 3: w[v] += v.ord + 1
for v in V.rev by 4: w[v] += 100
print(w)
print([v for v in V by 2], [v for v in V.rev by 2])
let k: Int = read()
for v in V by k: print(v)
