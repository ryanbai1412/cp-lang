import sys
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n, d = int(next(it)), int(next(it)); ms = [(int(next(it)), int(next(it))) for _ in range(n - 1)]
    best = [0] * (n + 1)
    def go(k, win, played):
        if k == n - 1:
            for a in range(1, n + 1): best[a] = max(best[a], played[a])
            return
        x, y = (win[p] if p > n else p for p in ms[k])
        played[x] += 1; played[y] += 1
        a, b = min(x, y), max(x, y)
        for w in ([a] if b - a >= d else [a, b]):
            win[n + 1 + k] = w; go(k + 1, win, played)
        played[x] -= 1; played[y] -= 1
    go(0, {}, [0] * (n + 1))
    print(*best[1:])
