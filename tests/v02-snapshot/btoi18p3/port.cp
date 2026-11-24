// Source: Baltic OI 2018 Day 1 P3 (Worm Worries), https://dmoj.ca/problem/btoi18p3
// Problem: interactive. An N x M x K box (each <= 1e6) of hidden humidities; "? x y z" asks one,
//   at most Q times; answer "! x y z" with a cell no lower than its up to six neighbours. Tests
//   are a line (N = 1e6, Q = 35), a sheet (N = M <= 1000, Q = 3500) or a box (<= 500^3).
// Solution: a line: golden-section search, which reuses one probe each round. A sheet: halve the
//   longer side along a line and take its highest cell c; if the best cell seen so far is
//   higher, recurse into its half; else c is the answer if both cells across the line are lower,
//   else recurse towards the higher one. Each step reads one line: O(N + M) queries. A box: best
//   of Q / 2 random cells, then walk uphill. Heights are memoised.
// original: corpus/submissions/4656860.cpp
let (n, m, k, q): (1..=1e6, 1..=1e6, 1..=1e6, 1..=1e5) = read()
domain X = 1..=n
domain Y = 1..=m
domain Z = 1..=k
type Cell = (X, Y, Z)

@memo                                               // WISH R10: a question asked twice is asked once
fn height(c: Cell) -> Int = ask("?", c, limit: q)   // WISH K21
fn beside(c: Cell) -> yields Cell = c.around(Dir6)  // WISH L1
fn peak(c: Cell) -> Bool = all(height(c) >= height(d) for d in beside(c))

// A sheet. Cut the longer side in half along a line and find the highest
// cell on the line. `top` is the highest cell seen on earlier lines; it lies
// inside the sheet, so a peak of the sheet is a peak of the grid.
type Sheet = Range<(X, Y)>  // WISH R11: a rectangle is a range of a product domain
fn on(p: (X, Y)) -> Int = height((p, Z.first))
fn on_sheet(sheet: Sheet, top: ?(X, Y)) -> (X, Y):
    let (line, low, high) = sheet.halve()  // WISH R11
    let c = argmax(line, by: on)
    if let t = top and on(t) >= on(c): return on_sheet(if t in low: low else: high, top)
    let (before, after) = line.across(c)  // WISH R11: the cells beside c in `low` and in `high`, if any
    let (below, above) = (if let p = before: on(p) else: 0, if let p = after: on(p) else: 0)
    if on(c) >= max(below, above): return c
    on_sheet(if below >= above: low else: high, c)

let found: Cell =
    if m == 1 and k == 1:
        // a line: golden-section search keeps one of its two probes for the next round
        (local_max(X, x => height((x, Y.first, Z.first))), Y.first, Z.first)  // WISH R10
    elif k == 1:
        (on_sheet((X, Y).all, None), Z.first)
    else:
        // a small box: start from the best of q / 2 random cells and walk uphill
        var c = argmax((Cell.random() for _ in 0..q / 2), by: height)
        while let d = first(d for d in beside(c) if height(d) > height(c)): c = d
        c
print("!", found)
