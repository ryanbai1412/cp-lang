// Source: Codeforces 2180G (Balance), https://codeforces.com/contest/2180/problem/G
// limits: 2000 ms, 256 MB
// Problem: an array starts empty; q <= 1e6 queries (multitest, sum q <= 1e6): 1 erases the
//   middle element, 2 x inserts x before, between and after all elements (L -> 2L + 1), 3
//   prints the sum over all 2^L - 1 non-empty subsequences b of (1 b_1 + ... + m b_m) / m,
//   mod 1e9 + 7.
// Solution: by symmetry the answer is ((2^L - 1) / L + 2^(L-1)) * sum / 2, so keep L, 2^L, sum
//   and the middle value mod p. The stack of type-2 layers is L in binary: 2 x pushes a 1 bit,
//   and an erase is L - 1, a binary decrement: the topmost odd layer loses its middle x
//   (becomes even, length 2L) and the layers above are replayed odd. Each decrement creates one
//   0 bit, so the replays are O(q) amortized in total.

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
        mid: None,
    )

fn total_value(st: State) -> Z = ((st.p2 - 1) / st.len + st.p2 / 2) * st.sum / 2

@!multitest
let q: 1..=1_000_000 = read()
var stk: List<State> = [State(0, 1, 0, 0, None)]

repeat q:
    let op: 1..=3 = read()
    match op:
        1 =>
            let l = (0..stk.len).rev.find(i => stk[i].mid != None)!
            stk[l] = double_even(stk[l - 1], stk[l].own)
            for j in l + 1..stk.len:
                stk[j] = double_odd(stk[j - 1], stk[j].own)
        2 =>
            let x: Z = read()
            stk.push(double_odd(stk.last!, x))
        3 =>
            print(total_value(stk.last!))
