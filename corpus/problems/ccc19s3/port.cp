// CCC 2019 S3 Arithmetic Square: a 3 x 3 grid of integers, some cells
// unknown (X). Fill the unknowns so every row and every column is an
// arithmetic progression. A filling always exists; any one is accepted.
//
// A line with exactly one unknown forces it. When nothing is forced, guess
// one cell and go on; a guess that leads to a half-integer middle is thrown
// away and the whole fill starts over, as in the original.
// original: corpus/submissions/4325116.cpp
type Square = List<List<?Int>>
const ROWS = [[(i, j) for j in 0..3] for i in 0..3]
const LINES = ROWS + [[(i, j) for i in 0..3] for j in 0..3]

fn read_cell() -> ?Int:
    let w: Str = read()
    if w == "X": None else: Int(w)

// Fill every forced cell. False when a middle would not be whole.
fn settle(mut sq: Square) -> Bool:
    var changed = true
    while changed:
        changed = false
        for line in LINES:
            let v = [sq[i][j] for (i, j) in line]
            if sum(1 for x in v if x == None) != 1: continue
            let (a, b, c) = (v[0] ?? 0, v[1] ?? 0, v[2] ?? 0)
            var x = 2 * b - a  // the third term
            var (i, j) = line[2]
            if v[0] == None:
                x = 2 * b - c
                (i, j) = line[0]
            elif v[1] == None:
                if (a + c) % 2 != 0: return false
                x = (a + c) / 2
                (i, j) = line[1]
            sq[i][j] = x
            changed = true
    true

fn attempt(given: Square) -> ?Square:
    var sq = given.copy()
    while true:
        if not settle(mut sq): return None
        let open = [(i, j) for i in 0..3 for j in 0..3 if sq[i][j] == None]
        if open.len == 0: return sq
        let (i, j) = open[rand(0, open.len - 1)]
        sq[i][j] = rand(-500, 499)

let given: Square = [[read_cell() for _ in 0..3] for _ in 0..3]
while true:
    let sq = attempt(given) else: continue
    for row in sq: print("{row[0]} {row[1]} {row[2]}")
    break
