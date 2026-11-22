//! Grids: a cell is the pair `(Row, Col)` and there is no grid type. `[Row, Col]T` is the grid,
//! and this module adds what a cell can do: `p.adj4`, `p.adj8`, `p.step(d)` and `p.ray(d)`. A
//! grid is a graph by writing its moves: `bfs(p => p.adj4.filter(q => g[q] != '#'), from: [s])`.

/// One of the four directions up, right, down, left (clockwise, rows growing downwards).
pub struct Dir4(k: Int):
    /// Up: one row less, `delta == (-1, 0)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Dir4.up().delta == (-1, 0)`
    pub fn up() -> Dir4 = todo
    /// Right: one column more, `delta == (0, 1)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Dir4.right().delta == (0, 1)`
    pub fn right() -> Dir4 = todo
    /// Down: one row more, `delta == (1, 0)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Dir4.down().delta == (1, 0)`
    pub fn down() -> Dir4 = todo
    /// Left: one column less, `delta == (0, -1)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Dir4.left().delta == (0, -1)`
    pub fn left() -> Dir4 = todo
    /// All four directions.
    ///
    /// cost: O(1)
    /// order: clockwise from up: up, right, down, left
    /// traps: never
    /// example: `[d.delta for d in Dir4.all()] == [(-1, 0), (0, 1), (1, 0), (0, -1)]`
    pub fn all() -> List<Dir4> = todo
    /// The direction a letter stands for, given the letters for up, right, down and left in
    /// that order, as in `Dir4.from("URDL", ch)` or `Dir4.from("NESW", ch)`. `None` for another
    /// letter; a repeated letter means the first direction it stands for.
    ///
    /// cost: O(1)
    /// traps: when `letters` is not four characters long
    /// example: `Dir4.from("URDL", 'D') == Some(Dir4.down())`
    /// example: `Dir4.from("NESW", 'x') == None`
    pub fn from(letters: Str, ch: Char) -> ?Dir4 = todo
    /// A quarter turn clockwise: up to right to down to left to up.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Dir4.up().cw == Dir4.right() and Dir4.left().cw == Dir4.up()`
    pub @get fn cw(self) -> Dir4 = todo
    /// A quarter turn counter-clockwise.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Dir4.up().ccw == Dir4.left()`
    pub @get fn ccw(self) -> Dir4 = todo
    /// The opposite direction: two quarter turns.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Dir4.left().back == Dir4.right()`
    pub @get fn back(self) -> Dir4 = todo
    /// The change in `(row, column)` of one step; rows grow downwards.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Dir4.down().delta == (1, 0)`
    pub @get fn delta(self) -> (Int, Int) = todo

/// One of the eight directions, clockwise from up: the four of `Dir4` and the diagonals between.
pub struct Dir8(k: Int):
    /// All eight directions.
    ///
    /// cost: O(1)
    /// order: clockwise from up: up, up-right, right, down-right, down, down-left, left, up-left
    /// traps: never
    /// example: `[d.delta for d in Dir8.all()] == [(-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1)]`
    pub fn all() -> List<Dir8> = todo
    /// An eighth of a turn clockwise.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Dir8.all()[0].cw == Dir8.all()[1] and Dir8.all()[7].cw == Dir8.all()[0]`
    pub @get fn cw(self) -> Dir8 = todo
    /// An eighth of a turn counter-clockwise.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Dir8.all()[0].ccw == Dir8.all()[7]`
    pub @get fn ccw(self) -> Dir8 = todo
    /// The opposite direction: four eighth turns.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Dir8.all()[1].back.delta == (1, -1)`
    pub @get fn back(self) -> Dir8 = todo
    /// The change in `(row, column)` of one step; each part is `-1`, `0` or `1`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Dir8.all()[3].delta == (1, 1)`
    pub @get fn delta(self) -> (Int, Int) = todo

/// The cell one step from `self` in direction `d` (a `Dir4` or a `Dir8`), or `None` when that
/// leaves the grid `R` by `C`.
///
/// cost: O(1)
/// traps: never
/// example: `let p: (0..3, 0..3) = (0, 0); p.step(Dir4.right()) == Some((0, 1)) and p.step(Dir4.up()) == None`
pub fn step<R, C, D>(self: (R, C), d: D) -> ?(R, C) = todo

/// The cells from `self` (not included) to the edge of the grid in direction `d`.
///
/// cost: O(1) to make, O(1) per cell
/// order: nearest first
/// traps: never
/// example: `let p: (0..3, 0..3) = (1, 0); p.ray(Dir4.right()).collect() == [(1, 1), (1, 2)]`
/// example: `let p: (0..3, 0..3) = (0, 0); p.ray(Dir4.up()).collect() == []`
pub fn ray<R, C, D>(self: (R, C), d: D) -> yields (R, C) = todo

/// The up to four cells inside the grid sharing a side with `self`.
///
/// cost: O(1) to make, O(1) per cell
/// order: up, right, down, left, skipping cells off the grid
/// traps: never
/// example: `let p: (0..3, 0..3) = (0, 1); p.adj4.collect() == [(0, 2), (1, 1), (0, 0)]`
pub @get fn adj4<R, C>(self: (R, C)) -> yields (R, C) = todo

/// The up to eight cells inside the grid sharing a side or a corner with `self`.
///
/// cost: O(1) to make, O(1) per cell
/// order: clockwise from up (`Dir8.all()` order), skipping cells off the grid
/// traps: never
/// example: `let p: (0..2, 0..2) = (0, 0); p.adj8.collect() == [(0, 1), (1, 1), (1, 0)]`
pub @get fn adj8<R, C>(self: (R, C)) -> yields (R, C) = todo

/// The grid with rows and columns exchanged: `t[c, r] == self[r, c]`.
///
/// cost: O(rows * cols)
/// empty: an empty grid gives an empty grid
/// order: keyed by `(C, R)`
/// traps: never
/// example: `let g: [0..2, 0..3]Int = [[1, 2, 3], [4, 5, 6]]; g.transposed() == [[1, 4], [2, 5], [3, 6]]`
pub fn transposed<R, C, T>(self: [R, C]T) -> [C, R]T = todo
