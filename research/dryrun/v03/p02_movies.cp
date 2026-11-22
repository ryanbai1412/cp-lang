// CSES 1629 Movie Festival: max number of non-overlapping [a, b) movies.
let n: 1..=2e5 = read()
let movies = read<[n](a: 1..=1e9, b: 1..=1e9)>()
var free_at = 0
var count = 0
for m in movies.sorted(by: m => m.b):
    if m.a >= free_at:
        count += 1
        free_at = m.b
print(count)
