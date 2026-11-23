// cf1486c2 Guessing the Greatest (hard): the second maximum s of the whole array tells which
// side of s the maximum is on; a binary search keeps s in the queried range.
let n: 2..=1e5 = read()
let wire = Wire()
fn ask(l: Int, r: Int) -> Int = wire.ask("? {l} {r}")

let s = ask(1, n)
let p = if s > 1 and ask(1, s) == s:
    (1..s).last_true(l => ask(l, s) == s)!
else:
    (s + 1..=n).first_true(r => ask(s, r) == s)!
print("!", p)

@interactor
fn judge(mut sol: Interactor):
    let n: Int = read()
    let a = read<[n]Int>()
    sol.send(n)
    var asked = 0
    while true:
        let c: Str = sol.recv()
        if c == "!":
            let p: Int = sol.recv()
            assert 1 <= p <= n and a[p - 1] == n
            return
        let l: Int = sol.recv()
        let r: Int = sol.recv()
        asked += 1
        assert c == "?" and asked <= 20 and 1 <= l < r <= n
        let order = (l..=r).sorted(by: i => a[i - 1], desc: true)
        sol.send(order[1])
