// Guess the number (interactive): the judge says n, hides x in 1..=n; "? y" answers ">=" when
// x >= y and "<" otherwise; at most 30 queries, then "! x".
let wire = Wire.new()
let n = read<1..=1e9>()
fn at_least(y: Int) -> Bool:
    let reply: Str = wire.ask("? {y}")
    reply == ">="
print("! {(1..=n).last_true(at_least)!}")
@interactor
fn judge(sol: Interactor):
    let (n, raw) = read<(1..=1e9, 1..=1e9)>()
    let secret = (raw - 1) % n + 1  // stress draws the two independently
    sol.send(n)
    var asked = 0
    while true:
        let kind: Str = sol.recv()
        let y: Int = sol.recv()
        if kind == "!":
            assert(y == secret)
            return
        asked += 1
        assert(asked <= 30)
        sol.send(if secret >= y: ">=" else: "<")
