"""CCO 2022 P3 inputs: `gen.py seed size` (few slides per room on a short clock, so moves race
the slides), or `gen.py --max` (3e5 slides per room over times up to 1e9, k near a slide)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])

def room(cnt, top):
    cuts = sorted(r.sample(range(top + 1), 2 * cnt))
    out = []
    for i in range(cnt):
        a, b = cuts[2 * i], cuts[2 * i + 1]
        if out and r.random() < 0.3:
            a = out[-1][1]
        out.append((a, b))
    r.shuffle(out)
    return out

if MAX:
    n1 = n2 = 300_000; top = 10**9; k = r.randint(1, 3000)
else:
    n1, n2 = r.randint(1, max(1, S)), r.randint(1, max(1, S))
    top = r.randint(2 * max(n1, n2), 2 * max(n1, n2) + r.choice([0, 5, 50]))
    k = r.randint(1, max(1, top // 3))
print(n1, n2, k)
for a, b in room(n1, top) + room(n2, top):
    print(a, b)
