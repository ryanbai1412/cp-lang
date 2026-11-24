let a = [3, 1]
print(a.sorted(cmp: (u, v) => cmp(u, v) then cmp(v, u)))
