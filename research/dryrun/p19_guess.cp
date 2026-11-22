// Guess the number (interactive): the judge says n, hides x in 1..=n; "? y" answers ">=" when
// x >= y and "<" otherwise; at most 30 queries, then "! x".
let n: 1..=1e9 = read()
// bsearch_last spends up to 31 queries on 1..=1e9 (repro/bsearch_query_count.cp)
var lo = 1
var hi = n
while lo < hi:
    let mid = lo + (hi - lo + 1) / 2
    let reply: Str = ask("?", mid, limit: 30)
    if reply == ">=": lo = mid else: hi = mid - 1
print("!", lo)

@interactor
fn judge(sol: Interactor):
    let (n, secret): (1..=1e9, 1..=1e9) = read()
    sol.send(n)
    while true:
        let kind: Str = sol.recv()
        let y: Int = sol.recv()
        if kind == "!":
            assert(y == secret)
            return
        sol.send(if secret >= y: ">=" else: "<")
