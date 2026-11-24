// CCC 2004 S5 Super Plumber: walk from the bottom-left to the bottom-right of the grid. Each step
// goes up, down or right; never revisit a cell and never enter `*`. Maximize the digits collected.
// Moving right is irreversible, so solve column by column: within a column you walk one straight
// run up or down from where you entered, then leave right.
fn value(ch: Char) -> Int = if '0' <= ch <= '9': ch - '0' else: 0

while true:
    let rows: 0..=100 = read()
    let cols: 0..=100 = read()
    if rows == 0 and cols == 0:
        break
    domain R = 0..rows
    domain Col = 0..cols
    let g: [R, Col]Char = read()
    // here[r]: the most collected on entering the column at row r.
    var here: [R]?Int = None
    here[R.last!] = Some(0)
    for c in Col:
        var leave: [R]?Int = None
        for from in R:
            let base = here[from] else: continue
            for step in [-1, 1]:
                var (q, got) = (from.ord, base)
                while q in 0..rows and g[R(q), c] != '*':
                    got += value(g[R(q), c])
                    leave[R(q)] max= got
                    q += step
        here = leave
    print(here[R.last!]!)
