let n: 1..=400_000 = read()
var ps = read<[n](Int, Int)>()
ps.sort()
print(ps[0].0 + ps[n - 1].1)
