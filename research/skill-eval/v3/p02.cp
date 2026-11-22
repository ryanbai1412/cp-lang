let n = read<1..=2e5>()
let movies = read<[n](a: 1..=1e9, b: 1..=1e9)>()
var free_from = 0
var watched = 0
for m in movies.sorted(by: m => m.b):
    if m.a >= free_from:
        watched += 1
        free_from = m.b
print(watched)
