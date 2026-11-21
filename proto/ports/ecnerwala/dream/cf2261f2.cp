// Codeforces 2261 F2: Plegma II: Ask Me Any Cell (Hard Version).
// To mark (r, c), every row is full of beacons except two: row r keeps only (r, c - 1),
// and the marker row r - 1 leaves columns 1 and 2 empty. The width stays n until the
// target row is alone, so S mod n tells the robot where it is: n - 2 or n - 1 while
// the marker row is inside (U), else 0 or 1 with whole rows left below (D); on row r
// alone S = 1 while (r, c - 1) is inside (L), then 0 (R). At most n + 2 zeros. Not
// proved here: that n + 2 is f(n) for n >= 10 (an accepted solution uses this scheme).
// For n = 3 it is not, so we print the sample answer.
enum Move: U, L, D, R

let n: 3 | 10..=40 = read()
if n == 3:
    let sample = ["111111001", "011111100", "110110110", "010111111", "111111111",
                  "010111101", "111111110", "111011111", "011010111"]
    for grid in sample:
        print(grid.chunks(3), sep: "\n")
        print()
    print("DURLDULRUU")
    return

domain Line = 1..=n
fn beacons(r: Line, c: Line) -> [Line, Line]Bool = [Line, Line] (i, j) =>
    if i == r: j == c - 1
    else: i != r - 1 or j > 2

fn decide(s: 0..=n * n) -> Move:
    if s % n >= n - 2: U
    elif s >= n: D
    elif s == 1: L
    else: R

for r in Line, c in Line:
    print(beacons(r, c), as: "01")
    print()
print([decide(s) for s in 0..=n * n].join())
