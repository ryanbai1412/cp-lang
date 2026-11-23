// expect: ok, output "? 7\n! 42"
// rule: 06.4.3
// tool: run {case}
// input: "7 41\n"
// has: "accepted the run"
// The interactor's `read()` reads the hidden test; `send` and `recv` talk to the program (decision 375).
let n: Int = read()
print("?", n)
flush()
let hidden: Int = read()
print("!", hidden + 1)

@interactor
fn judge(mut sol: Interactor):
    let (n, hidden): (Int, Int) = read()
    sol.send(n)
    let q: Str = sol.recv()
    let asked: Int = sol.recv()
    assert q == "?" and asked == n
    sol.send(hidden)
    let a: Str = sol.recv()
    let said: Int = sol.recv()
    assert said == hidden + 1
