"""gen.py SEED SIZE | --max: CCO 2018 P1. n <= 1000 games per team: a W/L string, then n scores
in 1..=1e6 (small cases use few distinct scores so ties occur). --max is n = 1000."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1000 if MAX else r.randint(1, 2 + 4 * S)
hi = 10 ** 6 if MAX or r.random() < .5 else 4
out = [str(n)]
for _ in range(2):
    out.append("".join(r.choice("WL") for _ in range(n)))
    out.append(" ".join(str(r.randint(1, hi)) for _ in range(n)))
sys.stdout.write("\n".join(out) + "\n")
