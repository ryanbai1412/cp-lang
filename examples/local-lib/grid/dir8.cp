//! Eight-way grid directions (`Dir8`), compatible with `std.grid.{step, ray}`.
use std.grid.{step, ray}

const DR = [-1, -1, 0, 1, 1, 1, 0, -1]
const DC = [0, 1, 1, 1, 0, -1, -1, -1]

/// One of the eight directions, clockwise from up; the odd ones are the diagonals.
pub struct Dir8(k: Int):
    /// All eight, clockwise from up.
    pub fn all() -> List<Dir8> = [Dir8(k) for k in 0..8]

    /// An eighth of a turn clockwise.
    pub @get fn cw(self) -> Dir8 = Dir8((k + 1) % 8)
    /// An eighth of a turn counter-clockwise.
    pub @get fn ccw(self) -> Dir8 = Dir8((k + 7) % 8)
    /// The opposite direction.
    pub @get fn back(self) -> Dir8 = Dir8((k + 4) % 8)
    /// The change in (row, column) of one step.
    pub @get fn delta(self) -> (Int, Int) = (DR[k], DC[k])

@test
fn dir8_turns_steps_and_rays():
    domain Row = 0..3
    domain Col = 0..5
    let p = (Row(1), Col(1))
    assert p.ray(Dir8(3)) == [(Row(2), Col(2))]
    assert p.step(Dir8(1)) == (Row(0), Col(2))
    assert p.step(Dir8(7)) == (Row(0), Col(0))
    for e in Dir8.all():
        assert e.cw.ccw == e and e.back.back == e and e.cw.cw.cw.cw == e.back
