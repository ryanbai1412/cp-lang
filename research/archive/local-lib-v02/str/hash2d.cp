//! Hashes of every axis-aligned rectangle of a grid, comparable in O(1): two rectangles
//! of the same shape with the same contents have the same hash, and different contents
//! collide with probability about `(rows + cols) / 2^31`.

use std.mod.{Mod}
type H = Mod<2147483647>

/// Prefix hashes of a grid with rows `R` and columns `C`. Building is O(R * C);
/// `rect` is O(1). The two bases are drawn at random when the structure is built, so
/// hashes are comparable only between rectangles of the same `Hash2D` (or two built
/// with the same `bases`).
pub struct Hash2D<R, C>(bases: (Int, Int)):
    /// `pre[i][j]` hashes the top-left `i` by `j` corner.
    pre: List<List<H>>
    pow_r: List<H>
    pow_c: List<H>

    /// `code(r, c)` is the cell as a number in `1..2^31 - 1` (a letter's code, a colour).
    pub fn new(code: (R, C) -> Int, bases: ?(Int, Int) = none) -> Hash2D<R, C>:
        let (br, bc) = bases ?? (rand(1 << 20, 1 << 30), rand(1 << 20, 1 << 30))
        var pre = [[H(0) for _ in 0..=C.K] for _ in 0..=R.K]
        for r in R:
            for c in C:
                let (i, j) = (r.to_idx, c.to_idx)
                pre[i + 1][j + 1] = pre[i][j + 1] * H(br) + pre[i + 1][j] * H(bc) - pre[i][j] * H(br) * H(bc) + H(code(r, c))
        var pow_r = [H(1)]
        for _ in R: pow_r.push(pow_r.last! * H(br))
        var pow_c = [H(1)]
        for _ in C: pow_c.push(pow_c.last! * H(bc))
        Hash2D((br, bc), pre, pow_r, pow_c)

    /// The hash of the cells in `rows` x `cols`.
    pub fn rect(self, rows: Range<R>, cols: Range<C>) -> Int:
        let (r1, r2, c1, c2) = (rows.start.ord, rows.end.ord, cols.start.ord, cols.end.ord)
        let (hr, hc) = (pow_r[r2 - r1], pow_c[c2 - c1])
        (pre[r2][c2] - pre[r1][c2] * hr - pre[r2][c1] * hc + pre[r1][c1] * hr * hc).v

@test
fn equal_hashes_exactly_for_equal_rectangles():
    domain R = 0..6
    domain C = 0..7
    for _ in 0..8:
        let g = [R, C] (r, c) => rand(1, 2)
        let h = Hash2D<R, C>((r, c) => g[r, c])
        for _ in 0..150:
            let (hgt, wid) = (rand(1, 3), rand(1, 3))
            let (r1, c1, r2, c2) = (
                rand(0, 6 - hgt),
                rand(0, 7 - wid),
                rand(0, 6 - hgt),
                rand(0, 7 - wid),
            )
            let same = all(
                g[R(r1 + i), C(c1 + j)] == g[R(r2 + i), C(c2 + j)] for i in 0..hgt for j in 0..wid
            )
            let a = h.rect(R(r1)..=R(r1 + hgt - 1), C(c1)..=C(c1 + wid - 1))
            let b = h.rect(R(r2)..=R(r2 + hgt - 1), C(c2)..=C(c2 + wid - 1))
            assert (a == b) == same

@test
fn find_a_pattern_in_a_grid():
    domain R = 0..4
    domain C = 0..5
    domain PR = 0..2
    domain PC = 0..2
    domain Letter = 'a'..='z'
    let text = ["abcab", "bcabc", "abbca", "cabab"]
    let pat = ["bc", "ab"]
    let big = Hash2D<R, C>((r, c) => Letter(text[r.to_idx][c.to_idx]).to_idx + 1)
    let small = Hash2D<PR, PC>(
        (r, c) => Letter(pat[r.to_idx][c.to_idx]).to_idx + 1,
        bases: big.bases,
    )
    let want = small.rect(PR(0)..=PR(1), PC(0)..=PC(1))
    var found: List<(Int, Int)> = []
    for r in 0..3:
        for c in 0..4:
            if big.rect(R(r)..=R(r + 1), C(c)..=C(c + 1)) == want: found.push((r, c))
    assert found == [(1, 0)]
