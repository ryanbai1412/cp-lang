var best = 0
var total = 0
for v in [3, 1, 4]:
    total += v
    best = max(total, v)
print(best, total)
