import sys
def value(a, k):
    f, seen, m = [], set(), 0
    for v in a:
        seen.add(v)
        while m in seen: m += 1
        f.append(m)
    return sorted(f, reverse=True)[k - 1]
def best(a, k):
    n = len(a); top = (value(a, k), -1)
    for i in range(n):
        for j in range(i + 1, n):
            b = a[:]; b[i], b[j] = b[j], b[i]
            if value(b, k) > top[0]: top = (value(b, k), (i + 1, j + 1))
    return top
def cases(text):
    it = iter(text.split()); t = int(next(it))
    for _ in range(t):
        n, k = int(next(it)), int(next(it)); yield [int(next(it)) for _ in range(n)], k
if __name__ == "__main__":
    for a, k in cases(sys.stdin.read()):
        v, p = best(a, k); print(-1 if p == -1 else f"{p[0]} {p[1]}")
