import sys, random
# seed size: m machines/steps, n jobs, a valid step order (job j appears m times), machines
# and times <= 20
r = random.Random(0 if sys.argv[1] == "--max" else int(sys.argv[1]))
S = 20 if sys.argv[1] == "--max" else int(sys.argv[2])
m = 20 if S >= 20 else r.randint(1, min(20, S + 1)); n = 20 if S >= 20 else r.randint(1, min(20, S + 1))
top = 20 if S >= 20 else r.choice([3, 20])
order = [j for j in range(1, n + 1) for _ in range(m)]; r.shuffle(order)
print(m, n); print(*order)
for _ in range(n): print(*[r.randint(1, m) for _ in range(m)])
for _ in range(n): print(*[r.randint(1, top) for _ in range(m)])
