import sys, random

if sys.argv[1:] == ["--max"]:
    # N = 5000 stations spread over the 1e6 blocks, random types, station 0 a C.
    r = random.Random(2014)
    n, M = 5000, 1_000_000
    blocks = sorted(r.sample(range(M), n))
    kinds = [r.randint(1, 2) for _ in range(n)]
    kinds[0], kinds[-1] = 1, 2
    z = r.choice([i for i in range(n) if kinds[i] == 1])
    rest = [i for i in range(n) if i != z]; r.shuffle(rest)
    order = [z] + rest
    print(n)
    print(" ".join(str(blocks[i]) for i in order))
    print(" ".join(str(kinds[i]) for i in order))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
M = 1_000_000
n = r.randint(1, S) if S < 1000 else S
span = r.choice([n, 2 * n, M]) if S < 1000 else r.choice([n, M])
span = min(max(span, n), M)
off = r.randint(0, M - span)
blocks = sorted(r.sample(range(span), n))
mode = r.randrange(3)
kinds = []
for i in range(n):
    if mode == 0: kinds.append(r.randint(1, 2))
    elif mode == 1: kinds.append(1 if i < n // 2 else 2)
    else: kinds.append(1 if r.random() < 0.3 else 2)
if n >= 2: kinds[0], kinds[-1] = 1, 2
else: kinds[0] = 1
# station 0 is a C; every C has a D to its right and every D a C to its left
cs = [i for i in range(n) if kinds[i] == 1]
z = r.choice(cs)
order = [z] + [i for i in range(n) if i != z]
rest = order[1:]; r.shuffle(rest); order = [z] + rest
print(n)
print(" ".join(str(off + blocks[i]) for i in order))
print(" ".join(str(kinds[i]) for i in order))
