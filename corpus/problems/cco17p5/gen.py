import sys, random
# `seed size`: n <= size people (size >= 2e5 gives n = 2e5), 0 <= A_i <= n, 0 <= B_i <= 1e4.
# Max seeds: all A_i = n with B_i = 1e4 (answer near 2e9, past 32 bits), or random.
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
N = 2 * 10 ** 5; seed = int(sys.argv[1])
n = N if S >= N else r.randint(1, S)
top = r.choice([0, 1, 10, 10 ** 4])
print(n)
for _ in range(n):
    if S >= N and seed % 3 == 0: print(n, 10 ** 4)
    elif S >= N and seed % 3 == 1: print(r.randint(n // 2, n), r.randint(0, 10 ** 4))
    else: print(r.randint(0, n), r.randint(0, top))
