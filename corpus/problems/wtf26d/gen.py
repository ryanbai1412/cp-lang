import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 10**6 if MAX else int(sys.argv[2])


def pair(n, mode):
    p = list(range(1, n + 1))
    if mode == 0:
        r.shuffle(p)
        q = list(range(1, n + 1)); r.shuffle(q)
    elif mode == 1:
        q = p[::-1]
    elif mode == 2:  # q shares a long prefix with p
        r.shuffle(p); q = p[:]
        i = r.randrange(n); tail = q[i:]; r.shuffle(tail); q[i:] = tail
    else:  # few adjacent swaps of p
        r.shuffle(p); q = p[:]
        for _ in range(r.randint(1, 5)):
            i = r.randrange(n - 1); q[i], q[i + 1] = q[i + 1], q[i]
    if q < p: p, q = q, p
    return p, q


if MAX:  # statement max: one case with N = sum N = 1e6, Q a few adjacent swaps of P (longest shared prefix)
    p, q = pair(S, 3)
    print("\n".join(["1", str(S), " ".join(map(str, p)), " ".join(map(str, q))]))
    sys.exit()
if S <= 1000:  # small: a few random cases with n <= S + 3
    ns = [r.randint(2, S + 3) for _ in range(5)]
else:  # large: sum N = S (statement: sum N <= 1e6), one big case or many small ones
    ns = [S] if r.random() < 0.6 else [2] * (S // 2)
mode = r.randrange(4)
out = [str(len(ns))]
for n in ns:
    p, q = pair(n, mode if S > 1000 else r.randrange(4))
    out += [str(n), " ".join(map(str, p)), " ".join(map(str, q))]
print("\n".join(out))
