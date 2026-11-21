// lsp: inlayHint, foldingRange, documentSymbol
// A comment run
// folds together.
domain Row = 0..2
enum Node:
    Src
    Cell(Row)
let n: 2..=10 = read()
let m = n * 3
fn f(k: Int) -> Int:
    let d = k % 7
    d + 1
for r in Row:
    print(f(n), m, r)
let ys = [1, 2].map(x =>
    let y = x + 1
    y
)
print(ys, Node.Src)
