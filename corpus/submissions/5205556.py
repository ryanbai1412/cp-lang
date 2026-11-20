import sys
if hasattr(sys, "set_int_max_str_digits"):
    sys.set_int_max_str_digits(1e9)

MOD = int(1e36 + 1412)


n, m = map(int, input().split())
arr = [int(input()) % MOD for _ in range(n + 1)]


def eval(x):
    ans = 0
    base = 1
    for c in arr:
        ans = (ans + base * c) % MOD
        base = (base * x) % MOD
    return ans


ans = []
for i in range(1, m + 1):
    if eval(i) == 0:
        ans.append(i)

print(len(ans))
for i in ans:
    print(i)