import sys, random
# `seed size`: n <= size ballots (size >= 1e4 gives n = 1e4, k = 26), each a permutation of the
# first k letters, made by a few random swaps of one base order so long councils exist.
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
N = 10 ** 4
n = N if S >= N else r.randint(1, S)
k = 26 if S >= N else r.randint(1, min(26, S + 2))
base = [chr(65 + i) for i in range(k)]; r.shuffle(base)
swaps = r.choice([0, 1, 2, k])
print(n, k)
for _ in range(n):
    p = base[:]
    for _ in range(r.randint(0, swaps)):
        i, j = r.randrange(k), r.randrange(k); p[i], p[j] = p[j], p[i]
    if S >= N and int(sys.argv[1]) % 3 == 0 and r.random() < 0.001: r.shuffle(p)
    print(''.join(p))
