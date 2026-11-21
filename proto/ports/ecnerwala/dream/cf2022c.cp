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
domain Col = 0..n
enum Row: Top, Bottom
let s: [Row][Col]Char = read()

fn other(r: Row) -> Row = if r == Row.Top: Row.Bottom else: Row.Top
fn won(cells: [3](Row, Col)) -> Int = Int(cells.count((r, c) => s[r][c] == 'A') >= 2)
fn bar(r: Row, c: Col) -> Int = won([(r, c), (r, (c + 1)!), (r, (c + 2)!)])
fn open_ell(z: Row, c: Col) -> Int = won([(z, c), (other(z), c), (other(z), (c + 1)!)])
fn close_ell(z: Row, c: Col) -> Int = won([(z, c), (z, (c + 1)!), (other(z), (c + 1)!)])

enum Front: Flush, Behind(Row)

@memo
fn best(c: Col.bounds, f: Front) -> Int = match f:
    Flush if c == Col.bounds.last => 0
    Flush =>
        let col = c.right!
        let stacked = bar(Row.Top, col) + bar(Row.Bottom, col) + best(c + 3, Front.Flush)
        let by_ell = max(open_ell(z, col) + best(c + 1, Front.Behind(z)) for z in Row)
        max(stacked, by_ell)
    Behind(z) =>
        let col = c.right!
        let close = close_ell(z, col) + best(c + 2, Front.Flush)
        if c.ord + 4 <= n:
            let stagger = bar(z, col) + bar(other(z), (col + 1)!) + best(c + 3, Front.Behind(z))
            max(close, stagger)
        else:
            close

print(best(Col.bounds.first, Front.Flush))
