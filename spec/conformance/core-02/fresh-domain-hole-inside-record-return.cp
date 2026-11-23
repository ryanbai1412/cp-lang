// expect: ok, output "3 2"
// rule: 02.9.3
// A `_` anywhere in the return type, here inside a record, is filled from the body's value.
struct S<V, P>:
    at: [P]V
fn mk<V>(r: V, k: Int) -> (s: S<V, _>, k: Int):
    domain P = 0..k
    (s: S([P] _ => r), k:)
domain V = 0..5
let m = mk(V(2), 3)
print(m.s.at.len, m.s.at[m.s.at.keys.first!].ord)
