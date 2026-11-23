let xs = [3, 1, 3, 2, 1, 3, 5, 5]
for r in xs.sorted().runs():
    print(r.len, r[0])
for (i, r) in xs.sorted(by: desc).runs().items:
    print(i, r.len, r[0])
