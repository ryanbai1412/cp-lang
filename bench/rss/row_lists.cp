// A row of lists made anew each round, as dmpg16g2 does: each round fills a
// different stretch of rows, so a buffer kept across rounds must not keep the
// lists it held before.
let n: 1..=100_000 = read()
let rounds: 1..=1000 = read()
domain Row = 0..n
var total = 0
for r in 0..rounds:
    var rows: [Row]List<Int> = []
    for i in 0..n: rows[Row((r * 1000 + i / 50) % n)].push(i)
    for row in rows if row.len > 0: total += row[0] + row.len
print(total)
