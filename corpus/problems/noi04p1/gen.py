"""noi04p1 generator: `n min`, then commands I k (k <= 1e5, at most 1e5), A k / S k (k <= 1000,
at most 100 in all) and F k (k >= 1, at most 1e5)."""
import random
import sys


def emit(floor, cmds):
    print(len(cmds), floor)
    print("\n".join(f"{c} {k}" for c, k in cmds))


if sys.argv[1] == "--max":
    rng = random.Random(0)
    cmds = [("I", rng.randint(0, 10**5)) for _ in range(10**5)]
    cmds += [("F", rng.randint(1, 10**5)) for _ in range(10**5)]
    cmds += [(rng.choice("AS"), rng.randint(0, 1000)) for _ in range(100)]
    rng.shuffle(cmds)
    emit(rng.randint(0, 10**5), cmds)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = rng.randint(1, max(1, size))
hi = rng.choice([20, 10**5])
cmds, adj = [], 0
for _ in range(n):
    c = rng.choice("IIFFAS" if adj < 100 else "IF")
    adj += c in "AS"
    k = rng.randint(0, min(hi, 1000)) if c in "AS" else rng.randint(1, max(2, n)) if c == "F" else rng.randint(0, hi)
    cmds.append((c, k))
emit(rng.randint(0, hi), cmds)
