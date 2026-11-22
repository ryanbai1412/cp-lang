# CCC 2004 S5 Super Plumber: walk from the bottom-left to the bottom-right of the grid. Each step
# goes up, down or right; never revisit a cell and never enter `*`. Maximize the digits collected.
# Moving right is irreversible, so solve column by column: within a column you walk one straight
# run up or down, then leave right.
loop:
    let (rows, cols) = read (Int, Int)
    if rows == 0 and cols == 0: break
    domain R = [rows]
    domain Col = [cols]
    let g = read [R, Col]Char

    let value = (r, c) => if g[r, c].is_digit(): Int(g[r, c].digit()) else: 0
    # here[r]: best collected so far when entering column c at row r.
    var here: [R]?Int = [R] r => if r == R.last: Some(0) else: None
    for c in Col:
        here = [R] r => if g[r, c] == '*': None else:
            max(for from in R if here[from].is_some() and R.span(from, r).all(q => g[q, c] != '*'):
                here[from]! + R.span(from, r).sum(of: q => value(q, c)))
    print(here[R.last]!)
