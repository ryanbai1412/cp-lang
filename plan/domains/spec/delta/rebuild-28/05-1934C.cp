// CF 1934C Find a Mine (interactive): corners (1,1), (1,m), (n,1) give two candidates on
// the anti-diagonal of the mine nearest (1,1); one more query decides between them.
let wire = Wire()

@interactor fn judge(mut sol: Interactor):
    let (n, m) = read<(Int, Int)>()
    let (x1, y1, x2, y2) = read<(Int, Int, Int, Int)>()
    sol.send("{n} {m}")
    var asked = 0
    while true:
        let r: Str = sol.recv()
        let x: Int = sol.recv()
        let y: Int = sol.recv()
        if r == "!":
            assert (x, y) == (x1, y1) or (x, y) == (x2, y2)
            return
        assert r == "?"
        assert 1 <= x <= n and 1 <= y <= m
        asked += 1
        assert asked <= 4
        sol.send(min(abs(x - x1) + abs(y - y1), abs(x - x2) + abs(y - y2)))

@!multitest
let (n, m) = read<(2..=1e8, 2..=1e8)>()

fn ask(x: Int, y: Int) -> Int = wire.ask("? {x} {y}")

let d1 = ask(1, 1)
let d2 = ask(1, m)
let d3 = ask(n, 1)
let sx = d1 + d2 - m + 3
let (ax, ay) = (sx / 2, d1 + 2 - sx / 2)
let sy = d1 + d3 - n + 3
let (bx, by) = (d1 + 2 - sy / 2, sy / 2)
if sx % 2 == 0 and 1 <= ax <= n and 1 <= ay <= m and ask(ax, ay) == 0:
    print("! {ax} {ay}")
else:
    print("! {bx} {by}")
