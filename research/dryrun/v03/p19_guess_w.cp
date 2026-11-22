// Guess the number (interactive): the judge says n, hides x in 1..=n; "? y" answers ">=" when
// x >= y and "<" otherwise; at most 30 queries, then "! x".
// p19 with print/read for the queries and a hand-written search (rows 15-17)
let n = read<1..=1e9>()
var lo = 1
var hi = n
while lo < hi:
    let mid = lo + (hi - lo + 1) / 2
    print("? {mid}")
    let reply = read<Str>()
    if reply == ">=": lo = mid else: hi = mid - 1
print("! {lo}")
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
