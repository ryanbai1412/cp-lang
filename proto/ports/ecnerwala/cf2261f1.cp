// Codeforces 2261 F1 (the F2 submission is the same program): print n*n grids
// and a move string. Grid (a, b) has a single 1 left of column b in row a, two
// leading 0s in the row above, and 1s everywhere else; n = 3 is a fixed table.
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
    for g in grids: print(g + "\n")
    print("DURLDULRUU")
    return

fn cell(a: Int, b: Int, i: Int, j: Int) -> Char:
    if i == a: return if j == b - 1: '1' else: '0'
    if i == a - 1 and j < 2: '0' else: '1'

for a in 0..n:
    for b in 0..n:
        for i in 0..n: print(Str(cell(a, b, i, j) for j in 0..n))
        print()

fn step(i: Int) -> Char:
    let c = i % n
    if c >= n - 2: return 'U'
    if i == 0: return 'R'
    if i == 1: return 'L'
    if c < 2: 'D' else: 'U'

print(Str(step(i) for i in 0..=n * n))
