import sys
def gap(b): return max(abs(x - y) for x, y in zip(b, b[1:]))
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n = int(next(it)); a = [int(next(it)) for _ in range(n)]
    print(min([gap(a)] + [gap(a[:l] + [2 * x for x in reversed(a[l:r])] + a[r:]) for l in range(n) for r in range(l + 1, n + 1)]))
