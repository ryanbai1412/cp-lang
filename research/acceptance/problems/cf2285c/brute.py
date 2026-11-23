import sys
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n = int(next(it)); a = tuple(int(next(it)) for _ in range(n))
    seen, todo = {a}, [a]
    while todo:
        b = todo.pop(); m = max(b)
        if b.count(m) != 1: continue
        i = b.index(m)
        for x in range(1, m):
            c = b[:i] + (x,) + b[i + 1:]
            if c not in seen: seen.add(c); todo.append(c)
    print(len(seen) % (10**9 + 7))
