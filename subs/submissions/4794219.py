n = int(input())
ans = len([0 for _ in range(n) if int(input()) * 5 - int(input()) * 3 > 40])
print(str(ans) + ("+" if ans == n else ""))