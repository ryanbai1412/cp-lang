// Codeforces 2022 C: Gerrymandering.
// Tile the 2 x n strip (3 | n) left to right. Every filled prefix ends either
// flush at cut c, best(c, none) (2(n - c) cells left, so 3 | (n - c) and
// c + 3 <= n whenever c < n), or behind, best(c, some(z)), with row z at c and
// other(z) at c + 1 (2(n - c) - 1 cells left, so c + 2 <= n always).
//   - Flush: two stacked bars advance to (c + 3, none); an opening L over c
//     leaves one row behind at (c + 1, some(z)).
//   - Behind(z): a closing L reaches (c + 2, none); two staggered bars reach
//     (c + 3, some(z)) whenever the bar on other(z) fits (`c + 4 <= n`).

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

@memo
fn best(c: 0..=n, behind: ?Row) -> Int:
    if let z = behind:
        let close = close_ell(z, c) + best(c + 2, none)
        if c + 4 <= n: max(
            close,
            bar(z, c) + bar(other(z), c + 1) + best(c + 3, some(z)),
        ) else: close
    elif c == n: 0
    else:
        let stacked = bar(Row.Top, c) + bar(Row.Bottom, c) + best(c + 3, none)
        max(stacked, max(open_ell(z, c) + best(c + 1, some(z)) for z in Row))
print(best(0, none))
