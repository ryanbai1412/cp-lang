old_input = input


def input():
    while True:
        s = old_input()
        if len(s):
            return s


n, m = map(int, input().split())
arr = [int(input()) for _ in range(n + 1)]


def eval(x):
    ans = 0
    base = 1
    for c in arr:
        ans += base * c
        base *= x
    return ans


ans = []
for i in range(1, m + 1):
    if eval(i) == 0:
        ans.append(i)

print(len(ans))
for i in ans:
    print(i)