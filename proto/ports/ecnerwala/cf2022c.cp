// Codeforces 2022 C: Gerrymandering.
// Tile the 2 x n strip (3 | n) left to right. Every filled prefix ends either
// Flush at cut c (2(n - c) cells left, so 3 | (n - c) and c + 3 <= n whenever
// c < n) or Behind(z) with row z at c and other(z) at c + 1 (2(n - c) - 1 cells
// left, so (n - c) % 3 == 2 and c + 2 <= n always).
//   - Flush: two stacked bars advance to (c + 3, Flush); an opening L over c
//     leaves one row behind at (c + 1, Behind(z)).
//   - Behind(z): a closing L reaches (c + 2, Flush); two staggered bars reach
//     (c + 3, Behind(z)) whenever the bar on other(z) fits (`c + 4 <= n`).

@!multitest
let n: 3..=100_000 = read()
enum Row: Top, Bottom
let s: [Row]Str = [Row] _ => read()
fn other(r: Row) -> Row = if r == Row.Top: Row.Bottom else: Row.Top
fn vote(r: Row, c: Int) -> Int = Int(s[r][c] == 'A')
fn won(a: Int, b: Int, c: Int) -> Int = Int(a + b + c >= 2)
fn bar(r: Row, c: Int) -> Int = won(vote(r, c), vote(r, c + 1), vote(r, c + 2))
fn open_ell(z: Row, c: Int) -> Int = won(vote(z, c), vote(other(z), c), vote(other(z), c + 1))
fn close_ell(z: Row, c: Int) -> Int = won(vote(z, c), vote(z, c + 1), vote(other(z), c + 1))

var flush: [0..=n]Int = 0
var lag: [Row][0..=n]Int = 0
for c in (0..n).rev:
    if (n - c) % 3 == 0:
        let stacked = bar(Row.Top, c) + bar(Row.Bottom, c) + flush[c + 3]
        flush[c] = max(stacked, max(open_ell(z, c) + lag[z][c + 1] for z in Row))
    elif (n - c) % 3 == 2:
        for z in Row:
            let close = close_ell(z, c) + flush[c + 2]
            lag[z][c] = if c + 4 <= n: max(
                close,
                bar(z, c) + bar(other(z), c + 1) + lag[z][c + 3],
            ) else: close
print(flush[0])
