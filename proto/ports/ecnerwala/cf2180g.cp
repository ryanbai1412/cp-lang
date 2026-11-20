const MOD = 1000000007
type Z = Mod<MOD>

struct State(len: Z, p2: Z, sum: Z, own: Z, mid: ?Z)

fn advance(st: State, v: Z, sub1: Bool) -> State:
    let two: Z = 2
    if not sub1:
        State(
            len: st.len + st.len + 1,
            p2: st.p2 * st.p2 * two,
            sum: st.sum + v * (st.len + 1),
            own: v,
            mid: some(st.mid ?? v),
        )
    else:
        State(
            len: st.len + st.len,
            p2: st.p2 * st.p2,
            sum: st.sum + v * (st.len + 1) - (st.mid ?? v),
            own: v,
            mid: none,
        )

@!multitest
let q: Int = read()
var stk: List<State> = [State(len: 0, p2: 1, sum: 0, own: 0, mid: none)]
let two: Z = 2
for _ in 0..q:
    let op: Int = read()
    if op == 1:
        var l = stk.len - 1
        while stk[l].mid == none: l -= 1
        stk[l] = advance(stk[l - 1], stk[l].own, true)
        for j in (l + 1)..stk.len:
            stk[j] = advance(stk[j - 1], stk[j].own, false)
    elif op == 2:
        let x: Z = read()
        stk.push(advance(stk.last!, x, false))
    else:
        let st = stk.last!
        print(((st.p2 - 1) / st.len + st.p2 / two) * st.sum / two)
