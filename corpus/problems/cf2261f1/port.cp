// Source: Codeforces 2261F1 (Plegma II: Ask Me Any Cell, Easy Version),
//   https://codeforces.com/contest/2261/problem/F1
// limits: 2000 ms, 256 MB
// Problem: given n (3 or 20), print n^2 binary n x n grids, the one for target (r, c) with
//   at most k zeros (k = 9, 24), and a table a_0..a_{n^2} of moves U/L/D/R: starting from the
//   whole grid, a robot makes 2n - 2 moves, each shrinking one side of its rectangle by the
//   entry a_S for S = ones inside it, and must end on exactly the target cell.
// Solution: n = 3 prints a precomputed answer. Otherwise grid (a, b) (0-based) zeroes row a
//   except a '1' gate at column b - 1 and zeroes columns 0..2 of row a - 1 (<= n + 1 zeros).
//   The table is R for S = 0, L for S = 1, else D when S mod n < 2 and U otherwise, so the
//   count of ones (missing ones mark the wall row and gate) steers the rectangle onto the
//   target. O(n^4) output.
let n: 3..=20 = read()  // the judge only has n = 3 and n = 20

if n == 3:
    let grids = [
        "111\n111\n001",
        "011\n111\n100",
        "110\n110\n110",
        "010\n111\n111",
        "111\n111\n111",
        "010\n111\n101",
        "111\n111\n110",
        "111\n011\n111",
        "011\n010\n111",
    ]
    for g in grids: print("{g}\n")
    print("DURLDULRUU")
    return

let blocked_cols = 0..2

fn cell(a: Int, b: Int, i: Int, j: Int) -> Char:
    if i == a: if j == b - 1: '1' else: '0'
    elif i == a - 1 and j in blocked_cols: '0'
    else: '1'

for a in 0..n:
    for b in 0..n:
        for i in 0..n: print(Str(cell(a, b, i, j) for j in 0..n))
        print()

fn step(i: Int) -> Char:
    if i == 0: 'R'
    elif i == 1: 'L'
    elif i % n < 2: 'D'
    else: 'U'

print(Str(step(i) for i in 0..=n * n))
