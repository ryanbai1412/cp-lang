// here: 3
// C++'s `INT_MAX` as a sentinel (self-run mistakes): "declare it first: `let INT_MAX = todo`".
var best = INT_MAX
for x in [3, 1, 2]:
    best = min(best, x)
print(best)
