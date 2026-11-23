let n: 1..=100 = read()
let xs: [0..n]Int = read()
for r in xs.sorted().runs():
    print(r.len, r[0])
for (i, r) in xs.sorted(by: desc).runs().items:
    print(i, r.len, r[0])
