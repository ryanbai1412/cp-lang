import sys
sys.setrecursionlimit(10000)
def composite(x): return x > 3 and any(x % d == 0 for d in range(2, int(x ** 0.5) + 1))
def match(n):
    own = [0] * (n + 1)
    def go(i, seen):
        for p in range(1, n + 1):
            if composite(i * p) and p not in seen:
                seen.add(p)
                if not own[p] or go(own[p], seen):
                    own[p] = i
                    return True
        return False
    if not all(go(i, set()) for i in range(1, n + 1)): return None
    perm = [0] * (n + 1)
    for p in range(1, n + 1): perm[own[p]] = p
    return perm[1:]
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    m = match(int(next(it)))
    print(-1 if m is None else " ".join(map(str, m)))
