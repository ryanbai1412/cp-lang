// Codeforces 2180 G: Balance (sequence doubling with midpoint deletion).
// Maintain a sequence starting empty under two mutations and a query:
//   - op 2 (x): interleave x into every gap (before, between, and after),
//     turning length L into 2L + 1, sum into sum + x * (L + 1), and keeping
//     the existing midpoint `mid` (or x when the sequence was empty).
//   - op 1: delete the current midpoint. Because doubling preserves the
//     midpoint of an odd-length sequence, deleting it rewinds to the highest
//     stack frame `l` that still carries a midpoint, switches that frame to
//     the midpoint-dropped doubling (`double_even`, length 2L, `mid = none`),
//     and replays the layers above `l` as `double_odd`.
//   - op 3: every element contributes equally by symmetry, so the answer for
//     length n (with `p2 = 2^n`) is `((2^n - 1) / n + 2^(n - 1)) * sum / 2`.

type Z = Mod<1_000_000_007>

struct State(len: Z, p2: Z, sum: Z, own: Z, mid: ?Z)

fn double_odd(st: State, v: Z) -> State =
    State(
        len: st.len + st.len + 1,
        p2: st.p2 * st.p2 * 2,
        sum: st.sum + v * (st.len + 1),
        own: v,
        mid: st.mid ?? v,
    )

fn double_even(st: State, v: Z) -> State =
    State(
        len: st.len + st.len,
        p2: st.p2 * st.p2,
        sum: st.sum + v * (st.len + 1) - (st.mid ?? v),
        own: v,
        mid: none,
    )

fn total_value(st: State) -> Z = ((st.p2 - 1) / st.len + st.p2 / 2) * st.sum / 2

@!multitest
let q: 1..=1_000_000 = read()
var stk: List<State> = [State(0, 1, 0, 0, none)]

repeat q:
    let op: 1..=3 = read()
    match op:
        1 =>
            let l = (0..stk.len).rev.first(i => stk[i].mid != none)!
            stk[l] = double_even(stk[l - 1], stk[l].own)
            for j in l + 1..stk.len:
                stk[j] = double_odd(stk[j - 1], stk[j].own)
        2 =>
            let x: Z = read()
            stk.push(double_odd(stk.last!, x))
        3 =>
            print(total_value(stk.last!))
