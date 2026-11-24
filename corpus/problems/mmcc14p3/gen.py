"""mmcc14p3 generator: n k, then n - 1 tree edges. --max: n = 2e5, a random tree."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 200_000 if big else r.randint(1, 2 + size * 3)
k = 50 if big else r.randint(1, n)
spread = r.choice([1, 3, n])
out = [f"{n} {k}"]
for v in range(2, n + 1):
    u = r.randint(max(1, v - spread), v - 1)
    out.append(f"{u} {v}" if r.random() < 0.5 else f"{v} {u}")
print("\n".join(out))
