//! Grids: a cell is the pair `(Row, Col)` and there is no grid type. `[Row, Col]T` is the grid,
//! and this module adds what a cell can do: `p.up` (and `down`, `left`, `right`), `p.adj4`,
//! `p.adj8`, `p.step(d)` and `p.ray(d)`. A
//! grid is a graph by writing its moves: `bfs(p => p.adj4.filter(q => g[q] != '#'), from: [s])`.

const DR = [-1, -1, 0, 1, 1, 1, 0, -1]
const DC = [0, 1, 1, 1, 0, -1, -1, -1]

/// One of the four directions up, right, down, left (clockwise, rows growing downwards).
pub struct Dir4(k: Int):
    /// Up: one row towards row 0, delta `(-1, 0)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Dir4.up.delta == (-1, 0)`
    pub const up: Dir4 = Dir4(0)
    /// Right: one column higher, delta `(0, 1)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Dir4.right.delta == (0, 1)`
    pub const right: Dir4 = Dir4(1)
    /// Down: one row higher (rows grow downwards), delta `(1, 0)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Dir4.down.delta == (1, 0)`
    pub const down: Dir4 = Dir4(2)
    /// Left: one column lower, delta `(0, -1)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Dir4.left.delta == (0, -1)`
    pub const left: Dir4 = Dir4(3)
    /// All four directions, clockwise from up. The list is fresh each call, but has a fixed
    /// length of four, so the cost is constant.
    ///
    /// cost: \(O(1)\)
    /// order: up, right, down, left
    /// traps: never
    /// example: `Dir4.all() == [Dir4.up, Dir4.right, Dir4.down, Dir4.left]`
    pub fn all() -> List<Dir4> = [Dir4(k) for k in 0..4]
    /// The direction a letter stands for, given the letters for up, right, down and left in that
    /// order: `Dir4.from("URDL", ch)`, `Dir4.from("NESW", ch)`. `None` for another letter.
    ///
    /// Matching is exact (case-sensitive). If a letter appears more than once in `letters`, the
    /// first position wins.
    ///
    /// cost: \(O(1)\)
    /// ties: a repeated letter maps to its first position in `letters`
    /// traps: `letters.len != 4`
    /// example: `Dir4.from("URDL", 'D') == Some(Dir4.down)`
    /// example: `Dir4.from("NESW", 'W') == Some(Dir4.left)`
    /// example: `Dir4.from("URDL", 'x') == None`
    pub fn from(letters: Str, ch: Char) -> ?Dir4:
        for k in 0..4:
            if letters[k] == ch: return Dir4(k)
        None
    /// A quarter turn clockwise: up to right, right to down, down to left, left to up.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Dir4.up.cw == Dir4.right`
    /// example: `Dir4.left.cw == Dir4.up`
    @get pub fn cw(self) -> Dir4 = Dir4((k + 1) % 4)
    /// A quarter turn counter-clockwise, the inverse of `cw`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Dir4.up.ccw == Dir4.left`
    /// example: `Dir4.down.cw.ccw == Dir4.down`
    @get pub fn ccw(self) -> Dir4 = Dir4((k + 3) % 4)
    /// The opposite direction, a half turn: its delta is the negated delta.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Dir4.up.back == Dir4.down`
    /// example: `Dir4.right.back == Dir4.left`
    @get pub fn back(self) -> Dir4 = Dir4((k + 2) % 4)
    /// The change in (row, column) of one step.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Dir4.all().map(d => d.delta) == [(-1, 0), (0, 1), (1, 0), (0, -1)]`
    @get pub fn delta(self) -> (Int, Int) = (DR[2 * k], DC[2 * k])

/// One of the eight directions, clockwise from up: the four of `Dir4` and the diagonals between.
pub struct Dir8(k: Int):
    /// All eight directions, clockwise from up: up, up-right, right, down-right, down,
    /// down-left, left, up-left. `Dir8` has no named constructors; index this list.
    ///
    /// cost: \(O(1)\)
    /// order: clockwise from up
    /// traps: never
    /// example: `Dir8.all().map(d => d.delta) == [(-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1)]`
    pub fn all() -> List<Dir8> = [Dir8(k) for k in 0..8]

    /// The direction whose letter is `ch` in `letters`, listed in `Dir8.all()` order, e.g.
    /// `Dir8.from("NOEKSJWH", ch)` style alphabets. `None` for another letter.
    ///
    /// Matching is exact (case-sensitive). If a letter appears more than once in `letters`, the
    /// first position wins.
    ///
    /// cost: \(O(1)\)
    /// ties: a repeated letter maps to its first position in `letters`
    /// traps: `letters.len != 8`
    /// example: `Dir8.from("abcdefgh", 'c') == Some(Dir8.all()[2])`
    /// example: `Dir8.from("abcdefgh", 'x') == None`
    pub fn from(letters: Str, ch: Char) -> ?Dir8:
        for k in 0..8:
            if letters[k] == ch: return Dir8(k)
        None
    /// An eighth of a turn clockwise: the next direction in `Dir8.all()`, wrapping from
    /// up-left to up.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Dir8.all()[0].cw.delta == (-1, 1)`
    /// example: `Dir8.all()[7].cw == Dir8.all()[0]`
    @get pub fn cw(self) -> Dir8 = Dir8((k + 1) % 8)
    /// An eighth of a turn counter-clockwise, the inverse of `cw`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Dir8.all()[0].ccw == Dir8.all()[7]`
    @get pub fn ccw(self) -> Dir8 = Dir8((k + 7) % 8)
    /// The opposite direction, four eighths round: its delta is the negated delta.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Dir8.all()[1].back.delta == (1, -1)`
    @get pub fn back(self) -> Dir8 = Dir8((k + 4) % 8)
    /// The change in (row, column) of one step; each part is `-1`, `0` or `1`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Dir8.all()[3].delta == (1, 1)`
    @get pub fn delta(self) -> (Int, Int) = (DR[k], DC[k])

/// The cell one row up from `self` (towards row 0), or `None` on the top row. A getter, like
/// `r.prev`: `if let q = p.up and black[q]: ...`.
///
/// `p.up` is [`step`] by the constant `Dir4.up`: one bounds test and one subtraction.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `domain R = 0..3; domain C = 0..3; (R(1), C(2)).up == Some((R(0), C(2)))`
/// example: `domain R = 0..3; domain C = 0..3; (R(0), C(2)).up == None`
@get pub fn up<R, C>(self: (R, C)) -> ?(R, C):
    if let r = self.0.prev: (r, self.1) else: None

/// The cell one row down from `self`, or `None` on the bottom row. A getter, like `r.next`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `domain R = 0..3; domain C = 0..3; (R(1), C(2)).down == Some((R(2), C(2)))`
/// example: `domain R = 0..3; domain C = 0..3; (R(2), C(2)).down == None`
@get pub fn down<R, C>(self: (R, C)) -> ?(R, C):
    if let r = self.0.next: (r, self.1) else: None

/// The cell one column left of `self`, or `None` in the first column. A getter, like `r.prev`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `domain R = 0..3; domain C = 0..3; (R(1), C(2)).left == Some((R(1), C(1)))`
/// example: `domain R = 0..3; domain C = 0..3; (R(1), C(0)).left == None`
@get pub fn left<R, C>(self: (R, C)) -> ?(R, C):
    if let c = self.1.prev: (self.0, c) else: None

/// The cell one column right of `self`, or `None` in the last column. A getter, like `r.next`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `domain R = 0..3; domain C = 0..3; (R(1), C(1)).right == Some((R(1), C(2)))`
/// example: `domain R = 0..3; domain C = 0..3; (R(1), C(2)).right == None`
@get pub fn right<R, C>(self: (R, C)) -> ?(R, C):
    if let c = self.1.next: (self.0, c) else: None

/// The cell one step from `self` in direction `d` (a `Dir4` or a `Dir8`), or `None` when that
/// step leaves the grid `(R, C)`.
///
/// `p.step(d)` is `(R, C).at((p.0 + d.delta.0, p.1 + d.delta.1))`: each coordinate must stay in
/// its own domain, so falling off one side never wraps to the other.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `domain R = 0..3; domain C = 0..3; (R(1), C(1)).step(Dir4.up) == Some((R(0), C(1)))`
/// example: `domain R = 0..3; domain C = 0..3; (R(0), C(1)).step(Dir4.up) == None`
/// example: `domain R = 0..3; domain C = 0..3; (R(1), C(1)).step(Dir8.all()[3]) == Some((R(2), C(2)))`
pub fn step<R, C, D>(self: (R, C), d: D) -> ?(R, C):
    let (dr, dc) = d.delta
    let r = R.at(self.0 + dr)
    let c = C.at(self.1 + dc)
    if r == None or c == None: None else: (r!, c!)

/// The cells from `self` (not included) to the edge of the grid in direction `d`, nearest
/// first: `p.step(d)`, then a step from that, until a step leaves the grid.
///
/// cost: \(O(1)\) to create, \(O(1)\) per cell
/// empty: yields nothing when `self.step(d)` is `None`
/// order: nearest first
/// traps: never
/// example: `domain R = 0..4; domain C = 0..4; (R(1), C(2)).ray(Dir4.down).collect() == [(R(2), C(2)), (R(3), C(2))]`
/// example: `domain R = 0..4; domain C = 0..4; (R(0), C(0)).ray(Dir8.all()[3]).collect() == [(R(1), C(1)), (R(2), C(2)), (R(3), C(3))]`
/// example: `domain R = 0..4; domain C = 0..4; (R(0), C(2)).ray(Dir4.up).collect() == []`
pub fn ray<R, C, D>(self: (R, C), d: D) -> yields (R, C):
    var p = step(self, d)
    while p != None:
        yield p!
        p = step(p!, d)

/// The up to four cells sharing a side with `self`: `self.step(d)` for each `d` in
/// `Dir4.all()`, skipping steps that leave the grid.
///
/// cost: \(O(1)\) to create, \(O(1)\) per cell
/// empty: yields nothing on a 1 × 1 grid
/// order: up, right, down, left, with off-grid cells skipped
/// traps: never
/// example: `domain R = 0..3; domain C = 0..3; (R(1), C(1)).adj4.collect() == [(R(0), C(1)), (R(1), C(2)), (R(2), C(1)), (R(1), C(0))]`
/// example: `domain R = 0..3; domain C = 0..3; (R(0), C(0)).adj4.collect() == [(R(0), C(1)), (R(1), C(0))]`
@get pub fn adj4<R, C>(self: (R, C)) -> yields (R, C):
    for d in Dir4.all():
        let q = step(self, d)
        if q != None: yield q!

/// The up to eight cells sharing a side or a corner with `self`: `self.step(d)` for each `d` in
/// `Dir8.all()`, skipping steps that leave the grid.
///
/// cost: \(O(1)\) to create, \(O(1)\) per cell
/// empty: yields nothing on a 1 × 1 grid
/// order: clockwise from up, with off-grid cells skipped
/// traps: never
/// example: `domain R = 0..3; domain C = 0..3; (R(1), C(1)).adj8.collect().len == 8`
/// example: `domain R = 0..3; domain C = 0..3; (R(0), C(0)).adj8.collect() == [(R(0), C(1)), (R(1), C(1)), (R(1), C(0))]`
@get pub fn adj8<R, C>(self: (R, C)) -> yields (R, C):
    for d in Dir8.all():
        let q = step(self, d)
        if q != None: yield q!

/// A new grid with rows and columns exchanged: `g.transposed()[c, r] == g[r, c]` for every
/// cell. The result is owned, not a view.
///
/// cost: \(O(\mathit{rows} \cdot \mathit{cols})\) time and space
/// empty: an empty grid gives an empty grid over `[C, R]`
/// order: keyed by `(C, R)`, so walking it is row-major over the new rows (the old columns)
/// traps: never
/// example: `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = [R, C] (r, c) => 10 * Int(r) + Int(c); g.transposed()[C(2), R(1)] == 12`
/// example: `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = [R, C] (r, c) => 10 * Int(r) + Int(c); g.transposed()[C(0)].collect() == [0, 10]`
pub fn transposed<R, C, T>(self: [R, C]T) -> [C, R]T = [C, R] (c, r) => self[r, c]

/// \(O(1)\). Every row of `g` as a view keyed by `R`: `g.rows[r]` is `g[r]` (R04b.1.12).
///
/// cost: \(O(1)\) to make, \(O(1)\) per row
/// empty: an empty view when `R` is empty
/// order: by `R`
/// traps: never
/// example: `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = 1; g.rows.len == 2`
@get pub fn rows<R, C, T>(self: [R, C]T) -> lends GridRows<R, C, T> = GridRows(self)

/// \(O(1)\). Every column of `g` as a view keyed by `C`: `g.cols[c]` is `g[.., c]` (R04b.1.12).
///
/// cost: \(O(1)\) to make, \(O(1)\) per column
/// empty: an empty view when `C` is empty
/// order: by `C`
/// traps: never
/// example: `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = 1; g.cols.len == 3`
@get pub fn cols<R, C, T>(self: [R, C]T) -> lends GridCols<R, C, T> = GridCols(self)

/// The rows of a grid, made by `g.rows`; lends the grid.
pub struct GridRows<R, C, T>(g: [R, C]T):
    /// \(O(1)\). How many rows: `R.count`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = 1; g.rows.len == 2`
    @get pub fn len(self) -> Int = R.count

    /// \(O(1)\). Whether there are no rows: `R` is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = 1; not g.rows.is_empty`
    @get pub fn is_empty(self) -> Bool = R.count == 0

    /// \(O(1)\). Row `r`, the same view as `g[r]`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = 1; g.rows[R(1)].sum() == 3`
    /// order: column order
    pub fn at(self, r: R) -> lends [C]T = g[r]

    /// The rows in `R` order.
    ///
    /// cost: \(O(1)\) per row
    /// empty: yields nothing
    /// order: by `R`
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = 1; g.rows.each().count() == 2`
    pub fn each(self) -> yields [C]T:
        for r in R: yield g[r]

/// The columns of a grid, made by `g.cols`; lends the grid.
pub struct GridCols<R, C, T>(g: [R, C]T):
    /// \(O(1)\). How many columns: `C.count`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = 1; g.cols.len == 3`
    @get pub fn len(self) -> Int = C.count

    /// \(O(1)\). Whether there are no columns: `C` is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = 1; not g.cols.is_empty`
    @get pub fn is_empty(self) -> Bool = C.count == 0

    /// \(O(1)\). Column `c`, the same view as `g[.., c]`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = 1; g.cols[C(1)].sum() == 2`
    /// order: row order
    pub fn at(self, c: C) -> lends [R]T = g[.., c]

    /// The columns in `C` order.
    ///
    /// cost: \(O(1)\) per column
    /// empty: yields nothing
    /// order: by `C`
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = 1; g.cols.each().count() == 3`
    pub fn each(self) -> yields [R]T:
        for c in C: yield g[.., c]

impl Show for GridRows<R, C, T>:
    fn show(v):
        var out = ""
        for x in v.each():
            if not out.is_empty: out += "\n"
            out += "{x}"
        out

impl Show for GridCols<R, C, T>:
    fn show(v):
        var out = ""
        for x in v.each():
            if not out.is_empty: out += "\n"
            out += "{x}"
        out
