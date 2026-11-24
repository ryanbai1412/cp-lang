import sys, random
# gen.py seed size: N <= 4 size + 4 countries in groups of S | N, every pair in a group plays once
# (shuffled, either side first, W/L/T), then 1 <= K <= S. --max: N = S = 2000, the most matches
# (1999000) in one group.
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
if MAX:
    n = s = 2000
else:
    n = r.randint(1, 4 * int(sys.argv[2]) + 4)
    s = r.choice([d for d in range(1, n + 1) if n % d == 0])
teams = list(range(1, n + 1)); r.shuffle(teams)
groups = [teams[i:i + s] for i in range(0, n, s)]
out = [f"{n} {s}"] + [" ".join(map(str, g)) for g in groups]
matches = [(g[i], g[j]) for g in groups for i in range(s) for j in range(i + 1, s)]
r.shuffle(matches)
for u, v in matches:
    if r.random() < 0.5: u, v = v, u
    out.append(f"{u} {v} {r.choice('WLT')}")
out.append(str(r.randint(1, s)))
print("\n".join(out))
