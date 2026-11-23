// expect: ok, output "6"
// rule: 01.7.1
// Each binder in an `if let` chain repeats `let`, and a later one may bind a tuple pattern from an optional pair.
let p: ?Int = Some(3)
let q: ?(Int, Int) = Some((1, 2))
if let x = p, let (a, b) = q:
    print(x + a + b)
