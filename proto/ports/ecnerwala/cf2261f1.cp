// Codeforces 2261 F1 (and F2): output `n * n` grids of size `n * n` and a walk
// of `n * n + 1` moves that distinguishes all `n * n` grids by their final
// reachable positions (each grid is tested from every open start cell).
//
// Construction for `n >= 4`:
// - Grid `(a, b)` places a horizontal wall of `'0'`s across row `a` with a
//   single `'1'` gate at column `b - 1` (no gate when `b == 0`), and blocks
//   `BLOCKED_COLS = 0..2` in row `a - 1`; all other cells are `'1'`.
// - The walk starts with `'R', 'L'` (probing `BLOCKED_COLS` in row `0`), then
//   runs `n` cycles of `n - 2` `'U'` steps and `2` `'D'` steps (`i % n < 2`),
//   ending with one final `'D'` at `i = n * n`.
// - Separation invariant: because `n - 2 >= 2`, each `'U'^(n-2)` phase resets
//   every column to the top of its vertical component (row `0` above the wall,
//   or row `a + 1` below it, except column `b - 1` which passes through the
//   gate). The `2` `'D'` steps and the `BLOCKED_COLS` notch in row `a - 1` then
//   leave row `a` and gate column `b - 1` uniquely encoded in the final position
//   distribution across the `n * n` grids.
// - Why `n = 3` is separate: at `n = 3`, `n - 2 = 1 < 2`, so `'U'` no longer
//   offsets `'D'^2` and the cycle drifts downward instead of resetting; the
//   9 grids and `"DURLDULRUU"` below are a precomputed solution found by search.
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
