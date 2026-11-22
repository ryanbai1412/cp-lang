domain V = 0..3
domain K = 1..=2
domain State = (V, K)

var dp: [State]Int = 0
for s in State:
    let (v, k) = s
    dp[s] = v.ord * 10 + k.label

print(dp[(V(2), K(1))], dp[V(2), K(2)], dp[State(V(1), K(2))])

var total = 0
for (v, k) in State:
    total += dp[(v, k)]
print(total, State.len)

fn advance(s: State) -> ?State:
    if let nv = s.0.next:
        (nv, s.1)
    else:
        none

let s1 = advance(State(V(0), K(2)))!
print(s1.0.ord, s1.1.label)

enum Dir = L | R
domain Named = (v: V, d: Dir, ok: Bool)
var seen: [Named]Int = 0
seen[(V(1), Dir.R, true)] = 42
let ns = Named(v: V(1), d: Dir.R, ok: true)
print(seen[ns], ns.v.ord, ns.d == Dir.R, ns.ok, Named.len)

var st = SegTree<State, Int>(+, fill: 0)
st.set((V(0), K(1)), 5)
st.set((V(2), K(2)), 7)
print(st.all)

fn local_prod(a: Int, b: Int) -> Int:
    domain A = 0..a
    domain B = 0..b
    domain P = (A, B)
    var t: [P]Int = 0
    for (x, y) in P:
        t[(x, y)] = x.ord * 10 + y.ord
    t[(A(a - 1), B(b - 1))]

print(local_prod(3, 5))
