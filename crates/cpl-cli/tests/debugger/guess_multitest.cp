@!multitest
let n: 1..=100 = read()
var lo = 1
var hi = n
while lo < hi:
    let mid = (lo + hi) / 2
    let r: Int = ask("?", mid, limit: 7)
    if r == 1: hi = mid
    else: lo = mid + 1
print("!", lo)

@interactor fn judge(sol: Interactor):
    let n: 1..=100 = read()
    let x: 1..=n = read()
    sol.send(n)
    var asked = 0
    while true:
        let (c, v): (Char, Int) = sol.recv()
        if c == '!':
            assert v == x, "wrong answer"
            return
        asked += 1
        assert asked <= 7, "too many queries"
        sol.send(if x <= v: 1 else: 0)
