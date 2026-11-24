// expect: ok, output "7 7 8"
// rule: 05.10.1
// `inner` captures only a copied `Int`, so it is an owned value and a plain `let`; the returned closure copies it in (decision 471).
fn mk(n: Int) -> (Int -> Int):
    let inner = (j: Int) => j * n
    (k: Int) => inner(k) + 1
let f = mk(3)
let g = (x: Int) =>
    let sq = (y: Int) => y * y
    sq(x) - x * 2
print(f(2), f(2), g(4))
