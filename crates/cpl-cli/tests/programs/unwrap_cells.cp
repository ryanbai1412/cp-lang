// `xs!` on `[D]?T` and `List<?T>` checks every cell in one pass (R02.4.8):
// `let at = at!` ends a fill phase in a body.
fn evens(n: Int) -> Int:
    var at: [0..n]?Int = None
    for i in 0..n: at[i] = i * 2
    let at = at!
    at[n - 1]

var xs: List<?Int> = [Some(1), Some(4)]
let ys = xs!
var zs: [1..=3]?Int = [Some(1), Some(2), Some(3)]
let ws = zs!
print(evens(3), ys.len, ys[1] + 1, ws[3])
