import sys
MOD = 998244353
def partitions(n):
    # every set partition of 0..n-1, as lists of blocks
    def rec(i, blocks):
        if i == n:
            yield blocks
            return
        for b in blocks:
            b.append(i); yield from rec(i + 1, blocks); b.pop()
        blocks.append([i]); yield from rec(i + 1, blocks); blocks.pop()
    yield from rec(0, [])
def solve(s):
    n = len(s); best, ways = -1, 0
    for p in partitions(n):
        if any(s[x] != s[y] and abs(x - y) != 1 for b in p for x in b for y in b):
            continue
        c = sum(len(b) * len({s[x] for x in b}) for b in p)
        if c > best: best, ways = c, 1
        elif c == best: ways += 1
    return best, ways % MOD
if __name__ == "__main__":
    it = iter(sys.stdin.read().split()); t = int(next(it))
    for _ in range(t):
        next(it); print(*solve(next(it)))
