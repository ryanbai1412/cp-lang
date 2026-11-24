let a = [3, 1]
var best = INT_MIN  // <- here
for x in a: best = max(best, x)
print(best)
