// Codeforces 1941D: Rudolf and the Ball Game
// I/O: t tests; each n m x, then m lines "r c" (c is 0 clockwise, 1 counterclockwise, ? either); print the count, then the sorted players who may hold the ball.
@!multitest
let n: 2..=1000 = read()
let m: 1..=1000 = read()
let x: 1..=1000 = read()
domain P = 0..n
var at: [P]Bool = false
at[P(x - 1)] = true
repeat m:
    let r: 1..=1000 = read()
    let c = read<Char>()
    var nxt: [P]Bool = false
    for p in P:
        if at[p]:
            if c != '1': nxt[P.wrap(Int(p) + r)] = true
            if c != '0': nxt[P.wrap(Int(p) - r)] = true
    at = nxt
let hold = [Int(p) + 1 for p in P if at[p]]
print(hold.len)
print(hold)
