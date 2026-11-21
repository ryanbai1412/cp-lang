// Codeforces 2022 C: Gerrymandering.
// Tile the 2 x n strip left to right. The filled part always ends flush at a
// column, or with one row a cell behind the other, so a DP over the frontier
// counts the most districts won right of it. Flush: two stacked bars stay
// flush, and an L over the column leaves one row behind. Behind: an L closes
// it to flush, and two staggered bars keep the same row behind.
@!multitest
let n: 3..=100_000 = read()
enum Row: Top, Bottom
let s: [Row]Str = [Row] _ => read()
fn other(r: Row) -> Row = if r == Row.Top: Row.Bottom else: Row.Top
fn vote(r: Row, c: Int) -> Int = Int(s[r][c] == 'A')
fn won(a: Int, b: Int, c: Int) -> Int = Int(a + b + c >= 2)
fn bar(r: Row, c: Int) -> Int = won(vote(r, c), vote(r, c + 1), vote(r, c + 2))
// The L over column c whose third cell sticks out in the row other than z.
fn ell(z: Row, c: Int) -> Int = won(vote(z, c), vote(other(z), c), vote(other(z), c + 1))

// flush[c]: both rows are filled left of c. lag[z][c]: row z is filled left
// of c and the other row left of c + 1. Cells left, 2(n - c) or 2(n - c) - 1,
// are a multiple of 3, so each frontier lives on its own residue of n - c.
var flush: [0..=n]Int = 0
var lag: [Row][0..=n]Int = 0
for c in (0..n).rev:
    if (n - c) % 3 == 0:
        let stacked = bar(Row.Top, c) + bar(Row.Bottom, c) + flush[c + 3]
        flush[c] = max(stacked, max(ell(z, c) + lag[z][c + 1] for z in Row))
    elif (n - c) % 3 == 2:
        for z in Row:
            let o = other(z)
            let close = won(vote(z, c), vote(z, c + 1), vote(o, c + 1)) + flush[c + 2]
            lag[z][c] = if c + 4 <= n: max(
                close,
                bar(z, c) + bar(o, c + 1) + lag[z][c + 3],
            ) else: close
print(flush[0])
