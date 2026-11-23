import sys
MOD = 998244353
def balanced(s):
    unstable = sum(1 for x in s if sum((x + d) in s for d in (1, 2, 3)) != 2)
    return unstable <= 2
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n = int(next(it)); a = [int(next(it)) for _ in range(n)]
    print(sum(balanced({a[i] for i in range(n) if m >> i & 1}) for m in range(1, 1 << n)) % MOD)
