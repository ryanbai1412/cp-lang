let a = [3, 1, 3, 2, 3]
var cnt = Counter<Int>(0)
for x in a:
    cnt.push(x)
print(cnt[3])
