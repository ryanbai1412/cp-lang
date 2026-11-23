import sys
def has(t, pat):
    it = iter(t)
    return all(c in it for c in pat)
it = iter(sys.stdin.read().split()); T = int(next(it))
for _ in range(T):
    n, s = int(next(it)), next(it)
    cnt = [0] * n
    for l in range(n):
        for r in range(l, n):
            sub = s[l:r + 1]
            if not (has(sub, "2025") and not has(sub, "2026")):
                for i in range(l, r + 1): cnt[i] += 1
    print(*cnt)
