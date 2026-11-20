// CCO 2018 Gradient Descent (interactive): k tokens lie on a rows x cols
// grid. "? r c" returns the total walking distance from (r, c) to all the
// tokens. Print "! d" for the least such total, using at most 170 questions.
let rows: 1..=1e7 = read()
let cols: 1..=1e7 = read()
let k: 1..=100 = read()
domain Row = 1..=rows
domain Col = 1..=cols

@memo                                                               // WISH K5: today this allocates rows * cols slots
fn cost(r: Row, c: Col) -> Int = ask("?", r, c, limit: 170)         // WISH K21: the budget next to the question

// cost(r, c) = f(r) + g(c) with f and g convex, so each axis is searched alone.
let r = argmin_convex(Row, r => cost(r, Col.first))                 // WISH K7
let c = argmin_convex(Col, c => cost(r, c))
print("!", cost(r, c))                                              // K2: today this prints "! ?" when the cell is new
