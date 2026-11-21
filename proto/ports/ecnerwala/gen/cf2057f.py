import random
import sys


def arr(r, n, top):
    a = [r.randint(1, top)]
    for _ in range(n - 1):
        a.append(r.randint(1, min(top, 2 * a[-1])))
    return a


def case(r, n, q, top, ktop):
    a = arr(r, n, top)
    ks = [r.randint(1, ktop) for _ in range(q)]
    return f"{n} {q}\n{' '.join(map(str, a))}\n" + "\n".join(map(str, ks)) + "\n"


if sys.argv[1] == "--max":
    r = random.Random(2057)
    n = q = 50_000
    a = [10**9 - r.randint(0, 5) for _ in range(n)]
    for i in range(0, n, 997):
        a[i] = r.randint(5 * 10**8, 10**9)
    ks = [r.randint(1, 10**9) for _ in range(q)]
    sys.stdout.write(f"1\n{n} {q}\n{' '.join(map(str, a))}\n" + "\n".join(map(str, ks)) + "\n")
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 3)
out = [str(t)]
for _ in range(t):
    n = r.randint(1, max(1, S // 3))
    q = r.randint(1, max(1, S // 3))
    top = r.choice([3, 20, 1000, 10**9])
    ktop = r.choice([5, 100, 10**4, 10**9])
    out.append(case(r, n, q, top, ktop))
sys.stdout.write("\n".join(out))
