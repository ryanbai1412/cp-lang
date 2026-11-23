// CF 2077B Finding OR Sum (interactive). Asking n forces n's bits on in both; what is left
// is, for every other bit i, c_i * 2^i with c_i = how many of x, y have bit i. Asking the
// even-bit mask leaves the odd bits two apart, so each c_i (0..=2) reads off as
// (rest >> i) & 3; the odd-bit mask gives the even bits the same way.
let wire = Wire()
let even = (0..30).filter(i => i % 2 == 0).map(i => 1 << i).sum()
let odd = (1 << 30) - 1 - even

@!multitest
let on_odd: Int = wire.ask("{even}")
let on_even: Int = wire.ask("{odd}")
let rest_odd = on_odd - 2 * even
let rest_even = on_even - 2 * odd
let m: 0..2 ** 30 = wire.ask("!")
var ans = 0
for i in 0..30:
    let c = (if i % 2 == 1: rest_odd else: rest_even) >> i & 3
    ans += (if m.bit(i): 2 else: c) << i
print(ans)

@interactor fn judge(mut sol: Interactor):
    let x, y, m: 0..2 ** 30 = read()
    var asked = 0
    loop:
        let s: Str = sol.recv()
        if s == "!": break
        asked += 1
        assert asked <= 2
        let n = s.parse_int()!
        sol.send((n | x) + (n | y))
    sol.send(m)
    let got: Int = sol.recv()
    assert got == (m | x) + (m | y)
