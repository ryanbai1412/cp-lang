let (n, m): (Int, Int) = read()
domain V = 0..n
domain E = 0..m
let deg: [V]Int = 0
for e in E:
    print(deg[e])
