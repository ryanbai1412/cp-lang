// CCO 2018 P4 Gradient Descent (interactive): the hidden function is a sum of |x - xi| + |y - yi|,
// so it separates into two convex functions. Minimize the row along one column, then the column
// along that row, by ternary search; remembering scores keeps within the query budget.
let rows: 1..=1e7 = read()
let cols: 1..=1e7 = read()
let k: 1..=170 = read()
domain Row = 1..=rows
domain Col = 1..=cols

@memo
fn score(r: Row, c: Col) -> Int:
    let s: 0..=2e9 = ask("?", r, c)
    s

let r = ternary_min(Row.first!..=Row.last!, i => score(i, Col.first!))!
let c = ternary_min(Col.first!..=Col.last!, j => score(r, j))!
print("!", score(r, c))
