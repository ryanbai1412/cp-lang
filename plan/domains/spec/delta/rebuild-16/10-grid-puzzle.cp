// Codeforces 1990D Grid Puzzle: whiten rows whose first a_i cells are black, using rows and 2x2s.
// In: t; per test n, then a_1..a_n. Out: the fewest operations per test.

// Which cells of the current row a 2x2 from the row above already whitened.
enum Covered = Nothing | Left | Right      // Left: columns 1-2, Right: columns 3-4

@!multitest
let n = read<1..=2e5>()
let a = read<[n]0..=n>()
var ops = 0
var cov: Covered = Nothing
for x in a:
    let (cost, next): (Int, Covered) = match cov:
        _ if x == 0 => (0, Nothing)
        _ if x > 4 => (1, Nothing)
        Nothing => if x <= 2: (1, Left) else: (1, Nothing)
        Left => if x <= 2: (0, Nothing) else: (1, Right)
        Right => (1, Left)
    ops += cost
    cov = next
print(ops)
