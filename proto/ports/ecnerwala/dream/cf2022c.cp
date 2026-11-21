// Codeforces 2022 C: Gerrymandering.
// Tile the 2 x n strip left to right. The filled part always ends flush at a
// column, or with one row a cell behind the other, so a DP over the frontier
// counts the most districts won right of it. Flush: two stacked bars stay
// flush, and an L over the column leaves one row behind. Behind: an L closes
// it to flush, and two staggered bars keep the same row behind.
@!multitest
let n: 3..=1e5 = read()
domain Col = 0..n
enum Row: Top, Bottom
let s: [Row][Col]Char = read()
fn won(cells: [3](Row, Int)) -> Int = Int(cells.count((r, c) => s[r][c] == 'A') >= 2)
fn bar(r: Row, c: Int) -> Int = won([(r, c), (r, c + 1), (r, c + 2)])

enum Front: Flush, Behind(Row)
@memo
fn best(c: Col.Cut, f: Front) -> Int = match f:
    Flush if c == Col.Cut.last => 0
    Flush => max(
        bar(Top, c) + bar(Bottom, c) + best(c + 3, Flush),
        max(won([(z, c), (! z, c), (! z, c + 1)]) + best(c + 1, Behind(z)) for z in Row),
    )
    Behind(z) => max(
        won([(z, c), (z, c + 1), (! z, c + 1)]) + best(c + 2, Flush),
        bar(z, c) + bar(! z, c + 1) + best(c + 3, Behind(z)) if c + 4 <= n,
    )
print(best(Col.Cut.first, Flush))
