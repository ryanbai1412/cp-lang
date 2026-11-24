// Source: CCC 2019 S3 (Arithmetic Square), https://dmoj.ca/problem/ccc19s3
// limits: 600 ms, 512 MB
// Problem: a 3 x 3 grid of integers in [-1e6, 1e6], some cells unknown (X). Fill the unknowns
//   so every row and column is an arithmetic sequence, all values in [-1e9, 1e9]; a solution
//   exists and any one is accepted.
// Solution: a line with exactly one unknown forces it (a middle is the mean of the ends), so
//   settle fills forced cells until none is left. When nothing is forced, set a random open cell
//   to a value in -500..499 and settle again. A forced middle that is not whole throws the
//   attempt away and the fill restarts from the input, as in the original.
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
    var sq = given.clone()
    while true:
        if not settle(mut sq): return None
        let open = [(i, j) for i in 0..3 for j in 0..3 if sq[i][j] == None]
        if open.len == 0: return sq
        let (i, j) = open[rand(0, open.len - 1)]
        sq[i][j] = rand(-500, 499)

let given: Square = [[read_cell() for _ in 0..3] for _ in 0..3]
while true:
    let sq = attempt(given) else: continue
    for row in sq: print("{row[0]!} {row[1]!} {row[2]!}")
    break
