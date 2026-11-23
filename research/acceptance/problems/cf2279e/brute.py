import sys
from collections import deque
def rank(a):
    basis = []
    for x in a:
        for b in basis: x = min(x, x ^ b)
        if x: basis.append(x)
    return len(basis)
def solve(a):
    # Operations are invertible linear maps over GF(2), so they keep the rank: full rank never reaches 0.
    if 0 in a: return 0
    if rank(a) == len(a): return -1
    n = len(a); s = tuple(a); seen = {s}; q = deque([(s, 0)])
    while q:
        u, d = q.popleft()
        for i in range(n - 1):
            x = u[i] ^ u[i + 1]
            if x == 0: return d + 1
            for v in (u[:i] + (x,) + u[i + 1:], u[:i + 1] + (x,) + u[i + 2:]):
                if v not in seen: seen.add(v); q.append((v, d + 1))
    return -1
if __name__ == "__main__":
    it = iter(sys.stdin.read().split()); t = int(next(it))
    for _ in range(t):
        n = int(next(it)); print(solve([int(next(it)) for _ in range(n)]))
