// Matched benchmark: noi20p1's walk, a user-built 250x250 max-plus Mat<?Int> squared 30 times,
// then a 1x250 row walked through those powers 200 times: the build stores the flat form a
// product reads, so no product converts its operand first.

domain V = 0..250
var s = 42
var grid: [V, V]?Int = [V, V] (_, _) => None
unchecked:
    for i in V:
        for _ in 0..3:
            s = (s * 1103515245 + 12345) & 2_147_483_647
            grid[i, V(s % 250)] = Some(s % 1000)
var pows = [Mat(grid, over: Semiring(max, +))]
for _ in 0..30: pows.push(pows.last! * pows.last!)
var row = Mat<?Int>.filled(1, V.count, None, over: Semiring(max, +))
row.set(0, 0, Some(0))
var acc = 0
for t in 0..200:
    s = (s * 1103515245 + 12345) & 2_147_483_647
    let e = s % 1_000_000 + 1
    for b in 0..30:
        if (e >> b) & 1 == 1: row = row * pows[b]
    acc = (acc + (row.get(0, t % 250) ?? 0) % 1_000_003) % 1_000_000_007
print(acc)
