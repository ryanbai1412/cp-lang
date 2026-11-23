var best = 0
var low = 9
var total = 0
for v in [3, 1, 4]:
    total += v
    best = max(total, v)
    if v < low: low = total
print(best, low, total)
