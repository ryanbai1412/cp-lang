// expect: ok, output "4 2\n0 1\n3 1"
// rule: 02.9.3
struct S<V, P>:
    at: [P]V
fn mk<V>(r: V, k: Int) -> S<V, _>:
    domain P = 0..k
    S<V, P>([P] _ => r)
domain V = 0..3
let s = mk(V(1), 4)
let t = mk(V(2), 2)
print(s.at.len, t.at.len)
for p in s.P:
    if p.ord == 0 or p.ord == 3: print(p, s.at[p])
