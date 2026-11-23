var total = 0
var cnt = [0, 0, 0]
for v in 0..3:
    total += v
    cnt[v] += 2
print(total, cnt[1])
