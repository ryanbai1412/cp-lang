// Codeforces 2261 F1 (and F2): output n*n grids of size n*n and a walk of
// n*n + 1 moves that distinguishes all n*n grids (n = 3 uses a fixed table).
// Grid (a, b) places a single '1' gate at (a, b - 1) inside an all-'0' row a,
// blocks columns 0..2 in row a - 1, and leaves all other cells '1'.
// For n >= 4, the walk starts with 'R', 'L' (probing columns 0..2 at row 0),
// then repeats cycles of length n: two 'D' steps (i % n < 2) followed by
// n - 2 'U' steps (i % n >= 2), ending with one final 'D' at i = n*n so each
// (a, b) pair is separated by which row and column gate blocks the oscillation.
let n: 3..=50 = read()

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

fn cell(a: Int, b: Int, i: Int, j: Int) -> Char:
    if i == a: if j == b - 1: '1' else: '0'
    elif i == a - 1 and j < 2: '0'
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
