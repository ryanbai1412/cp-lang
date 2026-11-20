n = int(input())
arr = [int(i) for i in input().split()]

size = 0
ans = 0
last = []
aans = []
for i in range(n):
    while size < arr[i]:
        ans += i - last.pop()
        size += 1
    if arr[i] == size:
        size += 1
    else:
        ans += size - arr[i] - 1
        last.append(i)
    aans.append(ans)
print(*aans)