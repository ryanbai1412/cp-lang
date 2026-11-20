// BOI 2018 Worm Worries (interactive): a box of n x m x k cells with hidden
// heights. "? x y z" gives one height, at most q times. Find a cell that is
// no lower than any of the (up to six) cells beside it. The tests are a
// line, a sheet, or a small box with a generous q.
// original: corpus/submissions/4656860.cpp
let (n, m, k, q): (1..=1e6, 1..=1e6, 1..=1e6, 1..=1e5) = read()
domain X = 1..=n
domain Y = 1..=m
domain Z = 1..=k
type Cell = (X, Y, Z)

@memo                                                               // WISH R10: a question asked twice is asked once
fn height(c: Cell) -> Int = ask("?", c, limit: q)                   // WISH K21
fn beside(c: Cell) -> yields Cell = c.around(Dir6)                  // WISH L1
fn peak(c: Cell) -> Bool = all(height(c) >= height(d) for d in beside(c))

// A sheet. Cut the longer side in half along a line and find the highest
// cell on the line. `top` is the highest cell seen on earlier lines; it lies
// inside the sheet, so a peak of the sheet is a peak of the grid.
type Sheet = Range<(X, Y)>                                          // WISH R11: a rectangle is a range of a product domain
fn on(p: (X, Y)) -> Int = height((p, Z.first))
fn on_sheet(sheet: Sheet, top: ?(X, Y)) -> (X, Y):
    let (line, low, high) = sheet.halve()                           // WISH R11
    let c = argmax(line, by: on)
    if let t = top and on(t) >= on(c): return on_sheet(if t in low: low else: high, top)
    let (before, after) = line.across(c)                            // WISH R11: the cells beside c in `low` and in `high`, if any
    let (below, above) = (if let p = before: on(p) else: 0, if let p = after: on(p) else: 0)
    if on(c) >= max(below, above): return c
    on_sheet(if below >= above: low else: high, c)

let found: Cell =
    if m == 1 and k == 1:
        // a line: golden-section search keeps one of its two probes for the next round
        (local_max(X, x => height((x, Y.first, Z.first))), Y.first, Z.first)     // WISH R10
    elif k == 1:
        (on_sheet((X, Y).all, none), Z.first)
    else:
        // a small box: start from the best of q / 2 random cells and walk uphill
        var c = argmax((Cell.random() for _ in 0..q / 2), by: height)
        while let d = first(d for d in beside(c) if height(d) > height(c)): c = d
        c
print("!", found)
