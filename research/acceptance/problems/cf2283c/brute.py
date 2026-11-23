import sys
def ok(n, d):
    k = m = 0
    for i in range(1, n + 1):
        k |= i
        m ^= i << d[i - 1]
    return m >= 2 * k
def solve(n):
    for mask in range(1 << n):
        d = [mask >> i & 1 for i in range(n)]
        if ok(n, d): return d
if __name__ == "__main__":
    ns = list(map(int, sys.stdin.read().split()))[1:]
    for n in ns: print(*solve(n))
