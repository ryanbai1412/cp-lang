ns = [131071, 65536]
rest = 200000 - sum(ns)
while rest > 0 and len(ns) < 10000:
    n = min(rest, [1, 2, 3, 4, 7, 8][len(ns) % 6])
    ns.append(n); rest -= n
print(len(ns))
for n in ns: print(n)
