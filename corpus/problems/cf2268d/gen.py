"""CF 2268 D inputs: `gen.py seed size`. size < 1e5 gives 5 random small permutations; size >= 1e5
gives a statement-max input (sum of n = 1e6) picked by seed % 5: one random n = 1e6; sorted
(one spine of n - 1 edges); reversed; a sawtooth of descending runs under rising peaks; or 1e4
tests of n = 100."""
import sys, random
if sys.argv[1] == "--max":  # sum n = 1e6: one n = 1e6 sawtooth (long descending runs under rising peaks)
    sys.argv[1:] = ["3", "1000000"]

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])


def saw(n):
    p, hi = [], 0
    while hi < n:
        run = min(n - hi, r.randint(1, 2000))
        p += list(range(hi + run, hi, -1)); hi += run
    return p


if S < 100000:
    tests = []
    for _ in range(5):
        n = r.randint(1, 4 * S); p = list(range(1, n + 1)); r.shuffle(p); tests.append(p)
else:
    n = 10**6; mode = int(sys.argv[1]) % 5
    if mode == 0:
        p = list(range(1, n + 1)); r.shuffle(p); tests = [p]
    elif mode == 1:
        tests = [list(range(1, n + 1))]
    elif mode == 2:
        tests = [list(range(n, 0, -1))]
    elif mode == 3:
        tests = [saw(n)]
    else:
        tests = []
        for _ in range(10000):
            p = list(range(1, 101)); r.shuffle(p); tests.append(p)
print(len(tests))
for p in tests:
    print(len(p)); print(*p)
