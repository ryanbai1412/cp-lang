// expect: ok, output "?\n! 8"
// rule: 10.2.10
// tool: run {case}
// has: "accepted the run"
// The interactor answers only once it has seen `?`, so the end check must flush it first.
let wire = Wire()
print("?")
let x = read<?Int>()
print("!", (x ?? 0) + 1)

@interactor
fn judge(mut sol: Interactor):
    let q: Str = sol.recv()
    assert q == "?"
    sol.send(7)
    let a: Str = sol.recv()
    let said: Int = sol.recv()
    assert said == 8
