// CCO 2018 Gradient Descent (interactive), as it builds today: the cache is a
// map keyed by one packed number, and each axis is a search on the slope (K1, K5, K7).
// original: corpus/submissions/4801866.cpp; judge: cco18p4_judge.py
let rows: 1..=1e7 = read()
let cols: 1..=1e7 = read()
let k: 1..=100 = read()
domain Row = 1..=rows
domain Col = 1..=cols

var seen: HashMap<Int, Int> = []
fn cost(r: Row, c: Col) -> Int:
    if let known = seen.get(r.label * (cols + 1) + c.label): return known
    let total: 0..=4e9 = ask("?", r, c)
    seen[r.label * (cols + 1) + c.label] = total
    total

fn row_turns(r: Row) -> Bool:
    if let s = r + 1: return cost(s, Col.first!) >= cost(r, Col.first)
    true
let r = Row.first_where(r => row_turns(r))!
fn col_turns(c: Col) -> Bool:
    if let d = c + 1: return cost(r, d) >= cost(r, c)
    true
let c = Col.first_where(c => col_turns(c))!
let best = cost(r, c)
print("!", best)
