// ccc04s5 Super Plumber: from the bottom-left to the bottom-right exit. In each column you
// enter, slide straight up or down (never through a `*`) collecting digits, then step right.
// Columns go right to left: best[r] is the most gold from entering this column at row r.
// One sweep per direction carries "the best from here on", so a column costs O(rows).
while true:
    let (rows, cols) = read<(0..=100, 0..=100)>()
    if rows == 0 and cols == 0:
        break
    domain R = 0..rows
    domain C = 0..cols
    let g = read<[R, C]Char>()
    var best: [R]?Int = None
    best[R.last!] = 0
    for c in C.rev:
        let gold = (r: R) => if g[r, c].is_digit: g[r, c].digit else: 0
        // slide(order)[r]: enter at r, slide back toward `order`'s start, step right.
        fn slide<S: Seq>(order: S) -> [R]?Int:
            var run: ?Int = None
            var out: [R]?Int = None
            for r in order:
                run = if g[r, c] == '*': None else: max(run, best[r]) + gold(r)
                out[r] = run
            out
        let (down, up) = (slide(R.range.rev), slide(R.range))
        best = [R] r => max(down[r], up[r])
    print(best[R.last!]!)
