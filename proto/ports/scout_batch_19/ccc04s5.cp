// CCC 2004 S5, Super Plumber — ideal CP Lang sketch
// Category: Unusual I/O (0 0 sentinel-terminated multi-test stream + mixed '.', '*', '1'..'9' grid)
// Original C++: corpus/submissions/4806811.cpp (90 LOC raw, ~52 LOC stripped)
// Buildable today-form: proto/ports/scout_batch_19/today/ccc04s5.cp (verified 40 random grids vs orig C++)

// WISH B19-6: tagged char-cell type parsing '.', '*', and digits '1'..'9' cleanly without manual ASCII math
@tags('.', '*', '1'..='9')
enum Cell:
    Empty
    Wall
    Coin(val: 1..=9)

// WISH B19-9: sentinel-terminated multi-test loop (`read_until((0, 0))`) instead of `while true` + `break`
for (rows, cols) in read_until((0, 0)):
    domain R = 0..rows
    domain C = 0..cols
    let grid: [R, C]Cell = read()

    // Column-by-column DP right-to-left: at column c, start at row r0, sweep down or up to r1,
    // then step right into dp[r1] of column c + 1. No grid transposition needed!
    var dp: [R]?Int = none
    dp[R.last] = 0

    for c in C.rev:
        var nxt: [R]?Int = none
        for start in R:
            for sweep in [start..R.end, (R.first..=start).rev]:
                var gain = 0
                for r in sweep:
                    match grid[r, c]:
                        Wall       => break
                        Empty      => pass
                        Coin(val)  => gain += val
                    if let after = dp[r]:
                        nxt[start] max= gain + after
        dp = nxt

    print(dp[R.last]!)

// WISH B19-3: constructive `gen` for sentinel-terminated multi-test streams guaranteeing at least
// one valid path from bottom-left (rows-1, 0) to bottom-right (rows-1, cols-1) (non-typeable reachability!)
gen cases:
    let t = rng.int(1..=min(size, 10))
    for _ in 0..t:
        let rows = rng.int(2..=min(size, 25))
        let cols = rng.int(2..=min(size, 25))
        var g: [0..rows, 0..cols]Char = '.'
        // Carve a guaranteed reachable path from (rows-1, 0) to (rows-1, cols-1) before placing walls
        var keep: [0..rows, 0..cols]Bool = false
        var r = rows - 1
        for c in 0..cols:
            let nr = if c == cols - 1: rows - 1 else: rng.int(0..rows)
            for k in min(r, nr)..=max(r, nr): keep[k, c] = true
            r = nr
        for i in 0..rows, j in 0..cols:
            if not keep[i, j] and rng.chance(0.25): g[i, j] = '*'
            elif rng.chance(0.35): g[i, j] = Char('1'.ord + rng.int(0..9))
        emit(rows, cols, g)
    emit(0, 0)
