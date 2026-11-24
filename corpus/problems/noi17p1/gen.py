import sys, random
# seed size: n <= size + 1 operations, b <= 30 n; a subtraction removes a term added earlier
# and still present, so x stays >= 0. --max: n = 1e6 with b up to 3e7
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[2] if mx else sys.argv[1]))
n = 10**6 if mx else r.randint(1, int(sys.argv[2]) + 1)
spread = 30 * n if mx or r.random() < 0.5 else min(30 * n, 64)
live, out = [], [f"{n} 3 3 3"]
for _ in range(n):
    t = r.random()
    if t < 0.45:
        a, b = r.choice([r.randint(1, 10**9), 1, 10**9]), r.randint(0, spread)
        live.append((a, b)); out.append(f"1 {a} {b}")
    elif t < 0.65 and live:
        i = r.randrange(len(live)); live[i], live[-1] = live[-1], live[i]
        a, b = live.pop(); out.append(f"1 {-a} {b}")
    else:
        out.append(f"2 {r.randint(0, min(30 * n, spread + 40))}")
print("\n".join(out))
