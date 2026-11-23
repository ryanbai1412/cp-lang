import sys, random
# `seed size`: n <= size + 1 (size >= 2e5 gives n = 2e5), skills up to 2e9, k up to 1e9.
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
N = 2 * 10 ** 5
n = N if S >= N else r.randint(1, S + 1)
top = r.choice([5, 10 ** 9, 2 * 10 ** 9]); k = r.randint(0, r.choice([0, 10, 10 ** 9]))
if S >= N and int(sys.argv[1]) % 2 == 0:
    top, k = 2 * 10 ** 9, 10 ** 9
print(n, k)
for _ in range(2): print(*[r.randint(max(1, top - r.choice([0, top - 1])), top) for _ in range(n)])
