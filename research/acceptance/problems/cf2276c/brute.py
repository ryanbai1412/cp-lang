import sys
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n, k = int(next(it)), int(next(it)); a = [int(next(it)) for _ in range(n)]
    print(sum(1 for l in range(n) for r in range(l + k, n) if sum(a[l:r + 1]) % k == 0))
