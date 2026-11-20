n = int(input())
arr = list(map(int, input().split()))
width = list(map(int, input().split()))
ans = 0
for i in range(n):
    ans += width[i] * (arr[i + 1] + arr[i]) / 2
print(ans)