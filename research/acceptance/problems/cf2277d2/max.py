import random
random.seed(1)
V = 10 ** 18
qs = [(0, V), (V, V), (0, 0), (1, V - 1)]
qs += [(2 ** k - 1, 2 ** (k + 1) - 2) for k in range(1, 60)]
qs += [(2 ** k - 2000, 2 ** k + 2000) for k in range(11, 60)]
while len(qs) < 10 ** 4:
    pick = random.random()
    if pick < 0.4: qs.append(tuple(sorted(random.randint(0, V) for _ in range(2))))
    elif pick < 0.7:
        r = random.randint(V // 2, V); qs.append((random.randint(0, r // 1000), r))
    else:
        k = random.randint(1, 59); x = min(max(2 ** k - random.randint(0, 2100), 0), V); qs.append((x, min(V, x + random.randint(0, 5000))))
print(len(qs))
for l, r in qs: print(l, r)
