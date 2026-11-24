"""gen.py SEED SIZE | --max: APIO 2023 P3 Alice, Bob, and Circuit, in the sample grader's format.
n members (0 <= n <= 700) with distinct names of 1 to 4 lowercase letters and favourite numbers
in [0, 65535], then m letters (0 <= m <= 1000) between members. --max is n = 700, m = 1000 with
4-letter names and numbers near 65535, so results wrap."""
import random, string, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 700 if MAX else r.randint(1, min(700, 1 + S))
m = 1000 if MAX else r.randint(0, min(1000, 2 * S))
names = set()
while len(names) < n:
    k = 4 if MAX else r.randint(1, 4)
    names.add("".join(r.choice(string.ascii_lowercase) for _ in range(k)))
names = list(names)
r.shuffle(names)
hi = 65535
nums = [r.randint(hi - 1000, hi) if MAX else r.randint(0, hi) for _ in range(n)]
print(n, m)
for s, x in zip(names, nums):
    print(s, x)
for _ in range(m):
    print(r.choice(names), r.choice(names))
