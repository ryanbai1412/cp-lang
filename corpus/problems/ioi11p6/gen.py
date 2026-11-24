"""gen.py SEED SIZE | --max: IOI 2011 Parrots, N then the message of N bytes. A seed draws
N in [1, min(64, 8 * SIZE)] and a message of random, all-equal, two-valued or sorted bytes;
--max is N = 64 random bytes (subtask 5's largest)."""
import random, sys

if sys.argv[1] == "--max":
    r, n, mode = random.Random(2011), 64, 0
else:
    r = random.Random(int(sys.argv[1]))
    n, mode = r.randint(1, min(64, 8 * int(sys.argv[2]))), r.randrange(4)
m = [r.randint(0, 255) for _ in range(n)]
if mode == 1:
    m = [m[0]] * n
elif mode == 2:
    m = [x & 1 for x in m]
elif mode == 3:
    m.sort()
print(n)
print(*m)
