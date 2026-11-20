//! Grids: a cell is the pair `(Row, Col)`, and its neighbours are named by how many there are.
//!
//! There is no grid type. `type Cell = (Row, Col)` indexes a `[Row, Col]T` directly, and
//! this module adds what a cell can do: `p.adj4`, `p.adj8`, `p.step(d)` and `p.ray(d)`.
//! Import the ones you use (`use std.grid.{adj4, Dir4, step}`); they are ordinary fns whose
//! first parameter is `self`, so `p.adj4` and `adj4(p)` are the same call.

const DR = [-1, -1, 0, 1, 1, 1, 0, -1]
const DC = [0, 1, 1, 1, 0, -1, -1, -1]

/// One of the four directions up, right, down, left (clockwise, rows growing downwards).
pub struct Dir4(k: Int):
    pub fn up() -> Dir4 = Dir4(0)
    pub fn right() -> Dir4 = Dir4(1)
    pub fn down() -> Dir4 = Dir4(2)
    pub fn left() -> Dir4 = Dir4(3)

    /// All four, clockwise from up.
    pub fn all() -> List<Dir4> = [Dir4(k) for k in 0..4]

    /// The direction a letter stands for, given the letters for up, right, down and left in
    /// that order: `Dir4.from("URDL", ch)`, `Dir4.from("NESW", ch)`. `none` for another letter.
    pub fn from(letters: Str, ch: Char) -> ?Dir4:
        for k in 0..4:
            if letters[k] == ch: return Dir4(k)
        none

    /// A quarter turn clockwise.
    pub @get fn cw(self) -> Dir4 = Dir4((k + 1) % 4)
    /// A quarter turn counter-clockwise.
    pub @get fn ccw(self) -> Dir4 = Dir4((k + 3) % 4)
    /// The opposite direction.
    pub @get fn back(self) -> Dir4 = Dir4((k + 2) % 4)
    /// The change in (row, column) of one step.
    pub @get fn delta(self) -> (Int, Int) = (DR[2 * k], DC[2 * k])

/// The cell one step from `self` in direction `d` (such as `Dir4`), or `none` off the grid.
pub fn step<R, C, D>(self: (R, C), d: D) -> ?(R, C):
    let (dr, dc) = d.delta
    let r = self.0 + dr
    let c = self.1 + dc
    if r == none or c == none: none else: (r!, c!)

/// The cells from `self` (not included) to the edge of the grid in direction `d`, nearest first.
pub fn ray<R, C, D>(self: (R, C), d: D) -> List<(R, C)>:
    var out: List<(R, C)> = []
    var p = step(self, d)
    while p != none:
        out.push(p!)
        p = step(p!, d)
    out

/// The up to four cells sharing a side with `self`: up, right, down, left.
pub @get fn adj4<R, C>(self: (R, C)) -> List<(R, C)>:
    var out: List<(R, C)> = []
    for d in Dir4.all():
        let q = step(self, d)
        if q != none: out.push(q!)
    out

/// The up to eight cells sharing a side or a corner with `self`, clockwise from up.
pub @get fn adj8<R, C>(self: (R, C)) -> List<(R, C)>:
    var out: List<(R, C)> = []
    for k in 0..8:
        let r = self.0 + DR[k]
        let c = self.1 + DC[k]
        if r != none and c != none: out.push((r!, c!))
    out

/// The grid with rows and columns exchanged.
pub @get fn transposed<R, C, T>(self: [R, C]T) -> [C, R]T = [C, R] (c, r) => self[r, c]

@test
fn neighbours_stay_on_the_grid():
    domain Row = 0..3
    domain Col = 1..=4
    type Cell = (Row, Col)
    let corner: Cell = (Row(0), Col(1))
    let mid: Cell = (Row(1), Col(2))
    assert corner.adj4 == [(Row(0), Col(2)), (Row(1), Col(1))]
    assert corner.adj8.len == 3 and mid.adj4.len == 4 and mid.adj8.len == 8
    for r in Row:
        for c in Col:
            let p: Cell = (r, c)
            let near = [
                (a, b) for a in Row for b in Col if (
                    a,
                    b,
                ) != p and abs(a.to_idx - r.to_idx) <= 1 and abs(b.to_idx - c.to_idx) <= 1
            ]
            assert sorted(p.adj8) == near
            assert sorted(p.adj4) == [
                q for q in near if abs(q.0.to_idx - r.to_idx) + abs(q.1.to_idx - c.to_idx) == 1
            ]

@test
fn directions_turn_and_step():
    domain Row = 0..3
    domain Col = 0..5
    let p = (Row(1), Col(1))
    let d = Dir4.from("URDL", 'R')!
    assert d == Dir4.right() and d.cw == Dir4.down() and d.ccw == Dir4.up() and d.back == Dir4.left()
    assert Dir4.from("NESW", 'x') == none
    assert p.step(d) == (Row(1), Col(2)) and p.step(Dir4.up())!.step(Dir4.up()) == none
    assert p.ray(d) == [(Row(1), Col(2)), (Row(1), Col(3)), (Row(1), Col(4))]
    // a grid is indexed by a cell, and a flood fill is three lines
    var open: [Row, Col]Bool = true
    open[Row(1), Col(3)] = false
    open[Row(0), Col(3)] = false
    open[Row(2), Col(3)] = false
    var seen: [Row, Col]Bool = false
    var todo = [p]
    seen[p] = true
    while let q = todo.pop():
        for n in q.adj4:
            if open[n] and not seen[n]:
                seen[n] = true
                todo.push(n)
    assert count(seen[r, c] for r in Row for c in Col) == 9

fn grid_sum<R, C>(g: [R, C]Int) -> Int = sum(g[r, c] for r in R for c in C)

@test
fn transposed_swaps_axes():
    domain Row = 0..2
    domain Col = 0..3
    let g = [Row, Col] (r, c) => 10 * r.to_idx + c.to_idx
    let t = g.transposed
    for r in Row:
        for c in Col: assert t[c, r] == g[r, c]
    assert grid_sum(g) == grid_sum(t)
    assert g.transposed.transposed == g
