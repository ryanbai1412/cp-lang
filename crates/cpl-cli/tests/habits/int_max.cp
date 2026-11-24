let a = [3, 1]
var best = LLONG_MAX  // <- here
for x in a: best = min(best, x)
print(best)
