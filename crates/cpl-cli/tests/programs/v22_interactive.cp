// A program that calls `ask` is interactive: every `print` goes out at
// once. Here the judge's answers are simply the rest of the input.
let n: Int = read()
print("range", n)
var lo = 1
var hi = n
while lo < hi:
    let mid = (lo + hi) / 2
    let answer: Str = ask("?", mid)
    if answer == "<": hi = mid
    else: lo = mid + 1
ask("!", lo)

// The judge's side is checked and is no part of the solution.
@interactor
fn judge(sol: Interactor):
    let (n, secret): (1..=1000, 1..=n) = read()
    sol.send(n)
    while true:
        let kind: Str = sol.recv()
        let x: Int = sol.recv()
        if kind == "!":
            assert x == secret
            return
        sol.send(if secret <= x: "<" else: ">=")
