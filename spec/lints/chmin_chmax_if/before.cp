var best = 0
var low = 9
var total = 0
for v in [3, 1, 4]:
    total += v
    best = max(best, v)
    if v < low: low = v
print(best, low, total)
