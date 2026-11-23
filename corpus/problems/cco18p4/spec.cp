// cco18p4 (interactive): a cell's score is the sum of Manhattan distances to the hidden tokens,
// a convex function of the row plus one of the column, so the best row does not depend on the
// column. Golden-section search (ternary_min) down column 1 for the best row, then along that
// row for the best column. Probes are reused and scores cached, so a 1e7 x 1e7 grid takes at most 68 questions (K >= 75).
let wire = Wire()
let rows: 1..=1e7 = read()
let cols: 1..=1e7 = read()
let k: 1..=170 = read()
domain Row = 1..=rows
domain Col = 1..=cols
type Score = 0..=2e9

// A memo may not do I/O (spec R06.4.1), so the asked scores are cached by hand.
var seen: HashMap<(Row, Col), Score> = []
fn score(r: Row, c: Col) -> Score:
    if let known = seen.get((r, c)): return known
    let s: Score = wire.ask("? {r} {c}")
    seen[(r, c)] = s
    s

let r = ternary_min(Row.range, i => score(i, Col.first!))!
let c = ternary_min(Col.range, j => score(r, j))!
print("!", score(r, c))
