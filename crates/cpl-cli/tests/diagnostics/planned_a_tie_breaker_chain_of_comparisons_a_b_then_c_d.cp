let a = [3, 1]
print(a.sort(cmp: (u, v) => u <=> v then v <=> u))
