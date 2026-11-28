// expect: ok, output "3 YES\n0 2 4\n2 2"
// rule: R02.9.3
// A binding's own fresh domain `s.P` is a domain through the same impls a declared one gets: it
// walks in order and steps, and two results' domains stay apart.
struct S<P>:
    pub at: [P]Int
fn make(k: Int) -> S<_>:
    domain P = 0..k
    S<P>([P] p => p.ord * 2)
let s = make(3)
let first = s.P.nth(0)
print(s.at.len, yes(first.next! == s.P.nth(1)))
print([s.at[p] for p in s.P])
let u = make(2)
print(u.at.len, s.at[first.next!])
