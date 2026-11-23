import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])


def perm(n, mode):
    p = list(range(1, n + 1))
    if mode == 0:
        r.shuffle(p)
    elif mode == 1:  # a few reversals of a sorted run: more solvable cases
        for _ in range(r.randint(0, min(n, 50))):
            i = r.randrange(n); j = r.randrange(n); i, j = min(i, j), max(i, j); p[i:j + 1] = p[i:j + 1][::-1]
    elif mode == 2:  # always solvable: build the deque backwards, each value at a random end
        from collections import deque
        d = deque()
        for v in range(n, 0, -1):
            (d.appendleft if r.random() < 0.5 else d.append)(v)
        p = list(d)
    else:
        p = p[::-1]
    return p


if S <= 1000:  # small: 8 cases with n <= S + 3
    ns = [r.randint(1, S + 3) for _ in range(8)]
else:  # large: sum N = S (statement: sum N <= 2.5e5), one big case or all n = 1
    ns = [S] if r.random() < 0.7 else [1] * S
mode = r.randrange(4)
out = [str(len(ns))]
for n in ns:
    out += [str(n), " ".join(map(str, perm(n, mode if S > 1000 else r.randrange(4))))]
print("\n".join(out))
