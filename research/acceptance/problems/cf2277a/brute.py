import sys
def best(n, k, a):
    top = 0
    def go(i, groups):
        nonlocal top
        if i == n:
            if all(len(g) >= k for g in groups):
                maxes = {max(g) for g in groups}
                m = 0
                while m in maxes: m += 1
                top = max(top, m)
            return
        go(i + 1, groups)
        for g in groups:
            g.append(a[i]); go(i + 1, groups); g.pop()
        groups.append([a[i]]); go(i + 1, groups); groups.pop()
    go(0, [])
    return top
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n, k = int(next(it)), int(next(it)); a = [int(next(it)) for _ in range(n)]
    print(best(n, k, a))
