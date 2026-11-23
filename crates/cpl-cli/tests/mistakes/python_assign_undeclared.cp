// here: 4
// Python's `best = 0` with no `let` (self-run mistakes): it was told `let best = todo`.
let a = [3, 1, 2]
best = 0
for v in a:
    best = max(best, v)
print(best)
