let n: 1..=1000 = read()
var rows: [0..n](name: Str, score: 0..=100) = read()
for (i, r) in rows.items:
    print("{i + 1}. {r.name} {r.score}")
