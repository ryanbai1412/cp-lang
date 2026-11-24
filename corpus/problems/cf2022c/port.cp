// Source: Codeforces 2022C (Gerrymandering), https://codeforces.com/contest/2022/problem/C
// limits: 2000 ms, 250 MB
// Problem: a 2 x n grid (3 | n, sum n <= 1e5) of A/J votes. Split it into connected districts
//   of 3 cells; a district goes to A if at least 2 of its cells vote A. Print the max number of
//   districts A wins. Multitest.
// Solution: tile left to right; every filled prefix ends Flush at cut c or Behind(z), row z at
//   c and the other row at c + 1. Flush: two stacked bars -> (c + 3, Flush), or an opening L
//   over column c -> (c + 1, Behind(z)). Behind(z): a closing L -> (c + 2, Flush), or two
//   staggered bars -> (c + 3, Behind(z)) when the bar on other(z) fits (c + 4 <= n).
//   Memoized best over (c, front): O(n).

@!multitest
let n: 3..=100_000 = read()
enum Row: Top | Bottom
let s: [Row]Str = [Row] _ => read()
fn other(r: Row) -> Row = if r == Row.Top: Row.Bottom else: Row.Top
fn vote(r: Row, c: Int) -> Int = Int(s[r][c] == 'A')
fn won(a: Int, b: Int, c: Int) -> Int = Int(a + b + c >= 2)
fn bar(r: Row, c: Int) -> Int = won(vote(r, c), vote(r, c + 1), vote(r, c + 2))
fn open_ell(z: Row, c: Int) -> Int = won(vote(z, c), vote(other(z), c), vote(other(z), c + 1))
fn close_ell(z: Row, c: Int) -> Int = won(vote(z, c), vote(z, c + 1), vote(other(z), c + 1))

enum Front: Flush | Behind(Row)

@memo
fn best(c: 0..=n, f: Front) -> Int = match f:
    Flush if c == n => 0
    Flush =>
        let stacked = bar(Row.Top, c) + bar(Row.Bottom, c) + best(c + 3, Front.Flush)
        let by_ell = [Row.Top, Row.Bottom].map(
            z => open_ell(z, c) + best(c + 1, Front.Behind(z))
        ).max()
        max(stacked, by_ell)
    Behind(z) =>
        let close = close_ell(z, c) + best(c + 2, Front.Flush)
        if c + 4 <= n:
            let stagger = bar(z, c) + bar(other(z), c + 1) + best(c + 3, Front.Behind(z))
            max(close, stagger)
        else:
            close

print(best(0, Front.Flush))
