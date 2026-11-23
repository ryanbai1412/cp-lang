// cf1934c Find a Mine: ask three corners. (1,1) and (1,m) pin one candidate,
// (1,1) and (n,1) another; one of them is always a mine, so check the first.
let wire = Wire()
@!multitest
let (n, m): (2..=1e8, 2..=1e8) = read()
fn ask(x: Int, y: Int) -> Int = wire.ask("? {x} {y}")
let d1 = ask(1, 1)
let d2 = ask(1, m)
let d3 = ask(n, 1)

let twice_x = d1 + d2 + 3 - m
if twice_x % 2 == 0:
    let x = twice_x / 2
    let y = d1 + 2 - x
    if x >= 1 and x <= n and y >= 1 and y <= m and ask(x, y) == 0:
        print("! {x} {y}")
        return
let y = (d1 + d3 + 3 - n) / 2
print("! {d1 + 2 - y} {y}")

@interactor
fn judge(mut sol: Interactor):
    let (n, m): (Int, Int) = read()
    let (x1, y1, x2, y2): (Int, Int, Int, Int) = read()
    sol.send("{n} {m}")
    var asked = 0
    while true:
        let c: Str = sol.recv()
        let x: Int = sol.recv()
        let y: Int = sol.recv()
        if c == "!":
            assert (x == x1 and y == y1) or (x == x2 and y == y2)
            return
        asked += 1
        assert c == "?" and asked <= 4 and x >= 1 and x <= n and y >= 1 and y <= m
        sol.send(min(abs(x - x1) + abs(y - y1), abs(x - x2) + abs(y - y2)))
