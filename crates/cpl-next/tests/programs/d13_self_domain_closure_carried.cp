// expect: ok, output "72"
// (spec) hand-evaluated: 0*10 + (4*10 + (3*10 + 2)); today prints 52 (its recursive-domain bug)
// Refusal probe: the local domain is hidden in a closure passed to the recursive call.
fn rec(n: Int, f: () -> Int) -> Int:
    if n == 0: return f()
    domain V = 0..n
    let b: [V]Int = n
    let g = () => V.count + 1
    f() * 10 + rec(n - 1, g)
print(rec(3, () => 0))
