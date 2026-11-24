# usage: gen.py SEED [SIZE]: n <= SIZE values in 1..=n, from few or many distinct values.
import random, sys
if sys.argv[1:] == ["--max"]:
    # n = 5e5 with about sqrt(n) distinct values in runs: many values with many copies, so
    # every value's blocks are built and swept
    g = random.Random(1)
    n = 5 * 10**5
    vals = [g.randint(1, 700) for _ in range(n)]
    print(n)
    print(*vals)
    sys.exit()

r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else r.choice([5, 12, 40, 300])
n = r.randint(1, size)
top = min(n, r.choice([1, 2, 3, max(1, n // 3), n]))
print(n)
print(*(r.randint(1, top) for _ in range(n)))
