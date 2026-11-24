"""gen.py SEED SIZE | --max: CCO 2007 P2 (Snowflakes). n <= 1e5 snowflakes of six arm lengths in
[0, 1e7), read around the flake from any arm in either direction. Small cases draw arms from a
tiny range and sometimes repeat a flake rotated or reflected. --max is n = 1e5 distinct flakes."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100_000 if MAX else r.randint(1, 1 + 2 * S)
hi = 9_999_999 if MAX or r.random() < 0.3 else r.randint(1, 3)
flakes, seen = [], set()
while len(flakes) < n:
    if flakes and not MAX and r.random() < 0.1:
        f = r.choice(flakes)
        k = r.randrange(6)
        f = f[k:] + f[:k]
        flakes.append(f[::-1] if r.random() < 0.5 else f)
        continue
    f = [r.randint(0, hi) for _ in range(6)]
    if MAX:
        key = min(min(g[k:] + g[:k] for k in range(6)) for g in (f, f[::-1]))
        if tuple(key) in seen:
            continue
        seen.add(tuple(key))
    flakes.append(f)
sys.stdout.write(f"{n}\n" + "".join(" ".join(map(str, f)) + "\n" for f in flakes))
