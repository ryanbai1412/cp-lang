// Guess the number (interactive): the judge says n, hides x in 1..=n; "? y" answers ">=" when
// x >= y and "<" otherwise; at most 30 queries, then "! x".
let n: 1..=1e9 = read()
fn at_least(y: Int) -> Bool:
    let reply: Str = ask("?", y, limit: 30)
    reply == ">="
print("!", bsearch_last(1..=n, at_least)!)

@interactor
fn judge(sol: Interactor):
    let (n, raw): (1..=1e9, 1..=1e9) = read()
    let secret = (raw - 1) % n + 1  // stress draws the two independently
    sol.send(n)
    while true:
        let kind: Str = sol.recv()
        let y: Int = sol.recv()
        if kind == "!":
            assert(y == secret)
            return
        sol.send(if secret >= y: ">=" else: "<")
