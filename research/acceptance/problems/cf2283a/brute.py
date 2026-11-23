import sys
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    l, r = int(next(it)), int(next(it)); a = list(range(l, r + 1)); n = len(a)
    keep = 0
    for m in range(1, 1 << n):
        s = [a[i] for i in range(n) if m >> i & 1]
        if sum(s) in s: keep = max(keep, len(s))
    print(n - keep)
