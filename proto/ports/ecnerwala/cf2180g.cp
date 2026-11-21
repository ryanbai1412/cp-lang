type Z = Mod<1_000_000_007>

struct State(len: Z, p2: Z, sum: Z, own: Z, mid: ?Z)

fn advance(st: State, v: Z, sub1: Bool) -> State:
    if not sub1:
        State(
            len: st.len + st.len + 1,
            p2: st.p2 * st.p2 * 2,
            sum: st.sum + v * (st.len + 1),
            own: v,
            mid: st.mid ?? v,
        )
    else:
        State(
            len: st.len + st.len,
            p2: st.p2 * st.p2,
            sum: st.sum + v * (st.len + 1) - (st.mid ?? v),
            own: v,
            mid: none,
        )

@! multitest
let q: Int = read()
var stk: [0..=q]State = State(0, 1, 0, 0, none)
var top = 0

for _ in 0..q:
    let op: Int = read()
    if op == 1:
        var l = top
        while stk[l].mid == none: l -= 1
        stk[l] = advance(stk[l - 1], stk[l].own, true)
        for j in l + 1..=top:
            stk[j] = advance(stk[j - 1], stk[j].own, false)
    elif op == 2:
        let x: Z = read()
        top += 1
        stk[top] = advance(stk[top - 1], x, false)
    else:
        let st = stk[top]
        print(((st.p2 - 1) / st.len + st.p2 / 2) * st.sum / 2)
