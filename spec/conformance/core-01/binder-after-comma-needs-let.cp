// expect: E0107 at 5
// rule: 01.7.1
// Each binder in a header repeats `let`: a bare `pat = value` after a comma is
// not CP Lang, however deep its parentheses go.
let p: ?Int = 1
let q: ?((Int, Int), Int) = ((2, 3), 4)
if let x = p, ((a, b), c) = q:
    print(x + a + b + c)
