//! Grids: a cell is the pair `(Row, Col)` and there is no grid type. `[Row, Col]T` is the grid,
//! and this module adds what a cell can do: `p.adj4`, `p.adj8`, `p.step(d)` and `p.ray(d)`. A
//! grid is a graph by writing its moves: `bfs(p => p.adj4.filter(q => g[q] != '#'), from: [s])`.

/// One of the four directions up, right, down, left (clockwise, rows growing downwards).
pub struct Dir4(k: Int):
    /// `(-1, 0)`. O(1).
    pub fn up() -> Dir4 = todo
    /// `(0, 1)`. O(1).
    pub fn right() -> Dir4 = todo
    /// `(1, 0)`. O(1).
    pub fn down() -> Dir4 = todo
    /// `(0, -1)`. O(1).
    pub fn left() -> Dir4 = todo
    /// All four, clockwise from up. O(1).
    pub fn all() -> List<Dir4> = todo
    /// The direction a letter stands for, given the letters for up, right, down and left in that
    /// order: `Dir4.from("URDL", ch)`, `Dir4.from("NESW", ch)`. `none` for another letter. O(1).
    pub fn from(letters: Str, ch: Char) -> ?Dir4 = todo
    /// A quarter turn clockwise. O(1).
    pub @get fn cw(self) -> Dir4 = todo
    /// A quarter turn counter-clockwise. O(1).
    pub @get fn ccw(self) -> Dir4 = todo
    /// The opposite direction. O(1).
    pub @get fn back(self) -> Dir4 = todo
    /// The change in (row, column) of one step. O(1).
    pub @get fn delta(self) -> (Int, Int) = todo

/// One of the eight directions, clockwise from up: the four of `Dir4` and the diagonals between.
pub struct Dir8(k: Int):
    /// All eight, clockwise from up. O(1).
    pub fn all() -> List<Dir8> = todo
    /// An eighth of a turn clockwise. O(1).
    pub @get fn cw(self) -> Dir8 = todo
    /// An eighth of a turn counter-clockwise. O(1).
    pub @get fn ccw(self) -> Dir8 = todo
    /// The opposite direction. O(1).
    pub @get fn back(self) -> Dir8 = todo
    /// The change in (row, column) of one step. O(1).
    pub @get fn delta(self) -> (Int, Int) = todo

/// O(1). The cell one step from `self` in direction `d` (a `Dir4` or a `Dir8`), or `none` off
/// the grid.
pub fn step<R, C, D>(self: (R, C), d: D) -> ?(R, C) = todo

/// O(1) to create, O(1) per cell: the cells from `self` (not included) to the edge of the grid
/// in direction `d`, nearest first.
pub fn ray<R, C, D>(self: (R, C), d: D) -> yields (R, C) = todo

/// O(1) to create: the up to four cells sharing a side with `self`: up, right, down, left.
pub @get fn adj4<R, C>(self: (R, C)) -> yields (R, C) = todo

/// O(1) to create: the up to eight cells sharing a side or a corner with `self`, clockwise from up.
pub @get fn adj8<R, C>(self: (R, C)) -> yields (R, C) = todo

/// O(rows · cols). The grid with rows and columns exchanged.
pub fn transposed<R, C, T>(self: [R, C]T) -> [C, R]T = todo
