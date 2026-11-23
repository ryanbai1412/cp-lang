let n: 1..=20 = read()
let xs: [0..n]1..=9 = read()
for r in xs.sorted().runs():
    print(r.first!, r.len)
for r in xs.sorted(by: desc).runs().items:
    print(r.0, r.1.len, r.1)
