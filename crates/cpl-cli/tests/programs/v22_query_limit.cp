// A solution that asks one question too many: `limit: 10` stops it at the
// eleventh, in the interpreter and in the C++ alike. `cpl run`
// runs the two against each other: the input is the hidden test, which
// only the judge reads, and what one side prints the other reads.
let n: Int = read()
var lo = 1
var hi = n
while lo < hi:
    let mid = lo
    // At most 10 queries find a number up to 1000.
    let answer: Str = ask("?", mid, limit: 10)
    if answer == "<": hi = mid
    else: lo = mid + 1
ask("!", lo)

// The judge's side is no part of the solution that `cpl build` writes.
@interactor
fn judge(mut sol: Interactor):
    let n: 1..=1000 = read()
    let secret: 1..=n = read()
    sol.send(n)
    while true:
        let kind: Str = sol.recv()
        let x: Int = sol.recv()
        if kind == "!":
            assert x == secret
            return
        sol.send(if secret <= x: "<" else: ">=")
