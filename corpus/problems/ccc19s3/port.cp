// Source: CCC 2019 S3 (Arithmetic Square), https://dmoj.ca/problem/ccc19s3
// limits: 600 ms, 512 MB
// Problem: a 3 x 3 grid of integers in [-1e6, 1e6], some cells unknown (X). Fill the unknowns
//   so every row and column is an arithmetic sequence, all values in [-1e9, 1e9]; a solution
//   exists and any one is accepted.
// Solution: a line with exactly one unknown forces it (a middle is the mean of the ends), so
//   settle fills forced cells until none is left. When nothing is forced, set a random open cell
//   to a value in -500..=499 (a seeded `Rng`) and settle again. A forced middle that is not whole throws the
//   attempt away and the fill restarts from the input, as in the original.
// original: corpus/submissions/4325116.cpp
use std.rand.{Rng}

domain Ax = 0..3
type Cell = (Ax, Ax)
// The rows and columns, each as its three cells in order.
let lines: List<(Cell, Cell, Cell)> = [((i, Ax(0)), (i, Ax(1)), (i, Ax(2))) for i in Ax] +
                                      [((Ax(0), j), (Ax(1), j), (Ax(2), j)) for j in Ax]

fn read_cell() -> ?Int:
    let w: Str = read()
    if w == "X": None else: Some(Int(w))
var given: [Ax, Ax]?Int = None
for i in Ax:
    for j in Ax: given[i, j] = read_cell()

// Fill every forced cell; false when a forced middle is not whole.
fn settle(mut sq: [Ax, Ax]?Int) -> Bool:
    var changed = true
    while changed:
        changed = false
        for (a, b, c) in lines:
            let (forced, value) = match (sq[a.0, a.1], sq[b.0, b.1], sq[c.0, c.1]):
                (None, Some(y), Some(z)) => (a, 2 * y - z)
                (Some(x), None, Some(z)) =>
                    if (x + z) % 2 != 0: return false
                    (b, (x + z) / 2)
                (Some(x), Some(y), None) => (c, 2 * y - x)
                _ => continue
            sq[forced.0, forced.1] = Some(value)
            changed = true
    true

var rng = Rng.new(2019)
fn attempt() -> ?[Ax, Ax]Int:
    var sq = given.clone()
    while settle(mut sq):
        let open = [(i, j) for i in Ax for j in Ax if sq[i, j].is_none]
        if open.is_empty: return Some([Ax, Ax] (i, j) => sq[i, j]!)
        let (i, j) = rng.pick(open)!
        sq[i, j] = Some(rng.int(-500, 499))
    None

var found = attempt()
while found.is_none: found = attempt()
print(found!)
